# Orgize

[![Crates.io](https://img.shields.io/crates/v/orgize.svg)](https://crates.io/crates/orgize)
[![Documentation](https://docs.rs/orgize/badge.svg)](https://docs.rs/orgize)
[![Build status](https://img.shields.io/github/actions/workflow/status/tao3k/orgize/ci.yml)](https://github.com/tao3k/orgize/actions/workflows/ci.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

Orgize is a Rust library for parsing Org mode documents. It keeps parsing
non-mutating by default: source blocks, links, agenda metadata, capture plans,
publishing graphs, and runtime-adjacent Org features are projected as
source-backed data instead of being executed.

The opt-in AOT parser is available to Cargo consumers without installing
Gerbil. Scheme declarations generate its syntax and Element tables, while a
transitional Rust structural scanner currently builds its lossless Rowan tree.
The complete Org recognition algorithm has not yet moved to Orgize's Scheme
event generator. The shipped artifacts support:

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

This is an opt-in parser surface while Org syntax coverage and the older
`Org::parse` consumers are being migrated. The contract pack is generated from
Orgize's Scheme declarations; its current evaluator is a Rust graph executor.

Named Element queries are declared in Orgize's `scheme :org-elements` blocks
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

The `org_contract_tangle` example likewise accepts optional Contract and
Element interface module paths for a consumer-owned generated Scheme source.
Omitting both paths preserves the upstream relative imports. This only
relocates imports; consumer Scheme admission, AOT generation and execution
still require their own qualification.

Live demo: <https://tao3k.github.io/orgize/>

## Parse

```rust
use orgize::{ast::ElementData, Org};

let org = Org::parse("* DONE Title :tag:");
let document = org.document();

assert_eq!(document.sections[0].level, 1);
assert_eq!(document.sections[0].raw_title, "Title ");
assert_eq!(document.sections[0].tags, ["tag"]);
assert!(document.sections[0].children.iter().all(|element| {
    !matches!(element.data, ElementData::Unknown { .. })
}));
```

Use `ParseConfig::parse` when a document needs custom parser settings:

```rust
use orgize::{syntax_ast::Headline, Org, ParseConfig};

let config = ParseConfig {
    todo_keywords: (vec!["TASK".to_string()], vec![]),
    ..Default::default()
};

let org = config.parse("* TASK Title 1");
let headline = org.first_node::<Headline>().unwrap();
assert_eq!(headline.todo_keyword().unwrap(), "TASK");
```

## Syntax Tree

Use `Org::syntax_document()` for the lossless rowan-backed syntax tree:

```rust
use orgize::{rowan::ast::AstNode, syntax_ast::Headline, Org};

let org = Org::parse("* Title");
let syntax_doc = org.syntax_document();
let headline = syntax_doc.syntax().children().find_map(Headline::cast).unwrap();

assert_eq!(headline.title_raw(), "Title");
```

## Traverse

```rust
use orgize::{
    export::{from_fn, Container, Event},
    Org,
};

let mut headline_count = 0;
let mut handler = from_fn(|event| {
    if matches!(event, Event::Enter(Container::Headline(_))) {
        headline_count += 1;
    }
});

Org::parse("* 1\n** 2\n*** 3\n****4").traverse(&mut handler);
assert_eq!(headline_count, 3);
```

## Modify

```rust
use orgize::{syntax_ast::Headline, Org, TextRange};

let mut org = Org::parse("hello\n* world");
let headline = org.first_node::<Headline>().unwrap();

org.replace_range(headline.text_range(), "** WORLD!");
let headline = org.first_node::<Headline>().unwrap();

assert_eq!(headline.level(), 2);
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
