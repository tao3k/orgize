# Orgize

[![Crates.io](https://img.shields.io/crates/v/orgize.svg)](https://crates.io/crates/orgize)
[![Documentation](https://docs.rs/orgize/badge.svg)](https://docs.rs/orgize)
[![Build status](https://img.shields.io/github/actions/workflow/status/tao3k/orgize/ci.yml)](https://github.com/tao3k/orgize/actions/workflows/ci.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

Orgize is a Rust library for parsing Org mode documents. It keeps parsing
non-mutating by default: source blocks, links, agenda metadata, capture plans,
publishing graphs, and runtime-adjacent Org features are projected as
source-backed data instead of being executed.

The public `Org::parse` facade and `parse_org_aot` use the Scheme-generated
Org event algorithm to build a lossless Rowan tree and Element graph. Cargo
consumers do not need Gerbil. The shipped artifacts support:

```rust
use orgize::org_aot::{org_contract_pack, parse_org_aot};

let document = parse_org_aot("* Evidence\n[[id:proof]]\n")?;
assert_eq!(document.syntax().to_string(), "* Evidence\n[[id:proof]]\n");
assert!(document.records().iter().any(|record| record.kind == "link"));
assert!(!org_contract_pack().rules.is_empty());
# Ok::<(), orgize::org_aot::OrgAotError>(())
```

Projected fields preserve their declared cardinality: `record.field("name")`
returns the first value, while `record.values("name")` iterates all values in
source order. For example, a planning line with both `SCHEDULED` and
`DEADLINE` exposes two separate `key` and `value` entries.

Some higher-level consumers still use the older Rust contract-registry
interpreter; that feature has not completed the Scheme-AOT cutover. The
Scheme-owned contract pack is generated from Orgize declarations and evaluated
over the Element graph by Rust.

## Python SDK

[`bindings/python`](bindings/python) contains the uv-managed `orgizepy` project.
Its three APIs are deliberately separate: `orgizepy.parser` parses raw Org
through the Scheme-AOT Rust/Rowan parser, `orgizepy.functions` exposes the
generated headline functions, and `orgizepy.contract` evaluates explicit
Element rows through the standalone Scheme/Gambit C ABI. The parser and
functions do not require a Gerbil runtime.

```python
from orgizepy.parser import parse_org
from orgizepy.functions import headline_functions

document = parse_org("#+TODO: NEXT DONE\n* NEXT Ship SDK\n")
headline = next(element for element in document.elements if element.kind == "headline")
assert headline_functions(document, headline.id).todo_keyword == "NEXT"
```

The Contract ABI is also independently consumable from C or Rust. It requires
one Gambit runtime initialization per process; see
[`bindings/c/include/orgize_standalone.h`](bindings/c/include/orgize_standalone.h).

Named Element queries are declared in Orgize's `scheme :org-elements-query` blocks
and compiled into a typed Rust pack at development time. Cargo consumers query
without Gerbil at build or runtime:

```rust
use orgize::org_aot::parse_org_aot;

let document = parse_org_aot("#+SEQ_TODO: WAIT | DONE\n* Team\n** WAIT Review\n")?;
let team = document.records().iter()
    .find(|record| record.kind == "headline" && record.field("title") == Some("Team"))
    .unwrap();
let open = document.query_named("tasks.open", team.id).unwrap();
assert_eq!(open.len(), 1);
# Ok::<(), orgize::org_aot::OrgAotError>(())
```

The query inherits file-local TODO declarations from the Element graph; users
do not duplicate them in the query. Property predicates compose with hygienic
`all-of` and `any-of` forms, which the Scheme module lowers to a bounded
disjunction of conjunctions before generating Rust. Each named query still
has one scope relation; negation, joins, ordering, and aggregation are not yet
admitted and have no implicit Rust fallback.
The exact `todo-keyword` predicate is also Scheme-AOT generated and checks the
document's own TODO declarations; it does not assume a global keyword list.

A consumer-owned example lives in
[`tests/fixtures/org-elements/customer-queries.org`](tests/fixtures/org-elements/customer-queries.org).
The `org_elements_tangle` example accepts an optional Element interface module
path, and the public Scheme AOT function emits a pack consumed by
`query_with_pack`. Authoring or regenerating that pack needs Gerbil in the
development environment; using its committed Rust artifact through Cargo does
not.

Contracts are a separate feature from Element queries. Define each in an Org
source block with its own Scheme feature header: `#+begin_src scheme :org-contract`
for assertions and `#+begin_src scheme :org-elements-query`
for named searches. The header is not a pseudo-language name, and plain Scheme
Babel blocks are neither feature. The
[`customer-contracts.org`](tests/fixtures/org-contract/customer-contracts.org)
fixture demonstrates a consumer-owned contract pack. `org_contract_tangle`
accepts optional Contract and Element interface module paths; omitting both
preserves upstream relative imports. During development,
`just generate-contract-plan PARSER_LIB POO_FLOW_LIB SOURCE.ss OUTPUT.rs`
admits the tangled POO declarations and emits a Rust pack. Cargo consumers
compile the committed pack and call `OrgAotDocument::evaluate_contract` without
installing Gerbil. This AOT path does not make host-loaded legacy contract
registries or CLI trace/capture use the generated pack yet.

Live demo: <https://tao3k.github.io/orgize/>

## Parse

```rust
use orgize::{ast::ElementData, Org};

let org = Org::parse("* DONE Title :tag:");
let document = org.document();

assert_eq!(document.sections[0].level, 1);
// raw_title keeps the source space before the tag suffix.
assert_eq!(document.sections[0].raw_title, "Title ");
assert_eq!(document.sections[0].tags, ["tag"]);
assert!(document.sections[0].children.iter().all(|element| {
    !matches!(element.data, ElementData::Unknown { .. })
}));
```

Use `ParseConfig::parse` when a document needs custom parser settings:

```rust
use orgize::ParseConfig;

let config = ParseConfig {
    todo_keywords: (vec!["TASK".to_string()], vec![]),
    ..Default::default()
};

let org = config.parse("* TASK Title 1");
let headline = &org.document().sections[0];
assert_eq!(headline.todo.as_ref().map(|todo| todo.name.as_str()), Some("TASK"));
```

## Syntax Tree

Use `Org::syntax()` for the lossless Scheme-AOT Rowan tree:

```rust
use orgize::Org;

let org = Org::parse("* Title");
assert_eq!(org.syntax().to_string(), "* Title");
assert_eq!(org.records().iter().filter(|record| record.kind == "headline").count(), 1);
```

## Query Elements

```rust
use orgize::Org;

let org = Org::parse("* 1\n** 2\n*** 3\n****4");
let headline_count = org.records().iter().filter(|record| record.kind == "headline").count();
assert_eq!(headline_count, 3);
```

## Modify

```rust
use orgize::{Org, TextRange};

let mut org = Org::parse("hello\n* world");
let headline = org.records().iter().find(|record| record.kind == "headline").unwrap().range;

org.replace_range(headline, "** WORLD!");
let headline = org.records().iter().find(|record| record.kind == "headline").unwrap().range;

assert_eq!(org.document().sections[0].level, 2);
org.replace_range(TextRange::up_to(headline.start()), "");
assert_eq!(org.to_org(), "** WORLD!");
```

## Documentation

The README is the crate entrypoint. Long-lived feature notes, parser surface
maps, release evidence, and architecture records live under `docs/`.

- `docs/index.org`: documentation coordinate index
- `docs/20_parser/20.05_parser_surface_map.org`: parser and semantic projection surface map
- <https://docs.rs/orgize>: public Rust API documentation

## Features

- `chrono`: timestamp integration
- `datafusion-sql`: in-process SQL over the stable `org_elements` table projection
- `indexmap`: indexmap-backed collections where enabled
- `md`: Markdown export support through `comrak`
- `syntax-org-fc`: syntax support for Org-fc-style use cases
