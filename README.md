# Orgize

[![Crates.io](https://img.shields.io/crates/v/orgize.svg)](https://crates.io/crates/orgize)
[![Documentation](https://docs.rs/orgize/badge.svg)](https://docs.rs/orgize)
[![Build status](https://img.shields.io/github/actions/workflow/status/tao3k/orgize/ci.yml)](https://github.com/tao3k/orgize/actions/workflows/ci.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

Orgize is a Rust library for parsing Org mode documents. It keeps parsing
non-mutating by default: source blocks, links, agenda metadata, capture plans,
publishing graphs, and runtime-adjacent Org features are projected as
source-backed data instead of being executed.

The public `Org::parse` facade and `parse_org_aot` execute the Org-owned Scheme
event strategy through the statically linked Gerbil AOT/FFI program, then build
the source-backed syntax index and Element graph. There is no Rust parser fallback.
Source builds currently require a prepared native program manifest and the
Gerbil toolchain; see the Justfile's `scheme-parser-build`,
`scheme-parser-stage` and `test` entries. Packaging and
release performance qualification are still pending.

Choose exactly one execution-owner feature. `runtime-rust` (default) owns
the request queue and thread-affine Gerbil handle in Rust. `runtime-scheme`
uses an in-process native service for initialization and a Scheme actor
consumer, without the Rust scheduling queue. Both use the same Scheme AOT
parser and Rowan/Element projection. Scheme still owns GC in both modes.
Select the latter with `default-features = false, features = ["runtime-scheme"]`.
The Scheme service currently requires POSIX threads, with one worker: it is
not a general green-thread I/O scheduler or proven SMP. Selecting both or
neither is a compile error; `--all-features` is intentionally invalid.
Both modes require explicit startup before application workers or children.
Call `unsafe { orgize::initialize_native_runtime() }` in an exclusive startup
window, with valid stdin/stdout/stderr, before using any native parser or
contract API. Parsing does not initialize implicitly. The first call captures
host control-signal dispositions and stdio flags, starts the selected native
owner, then restores host resources before admitting requests. Later calls
return the cached result; initialization failure is terminal, not retried.
The host must not create children or concurrently use stdio/change signal
dispositions during the first call. The linked parser must not use Scheme-owned
stdio, terminals or subprocesses. Native heartbeat/processor signals remain
runtime-owned. This is a POSIX startup contract, not general signal isolation,
restart, dynamic unload or unrestricted embedding in an already-running host.

Use `just check runtime-rust`, `just test runtime-rust` and
`just bench runtime-rust` (or `runtime-scheme`). These call Cargo directly.
Set a nondefault native manifest once through
`ORGIZE_GERBIL_PROGRAM_MANIFEST`; the bridge owns SDK/compiler discovery.
Workspace optimization profiles remain unchanged. Native library startup is
isolated from pure libtest cases because first initialization owns host stdio.

Criterion owns sampling, calibration, baselines and reports. Existing warm
fixture groups retain 1/2/4/8 callers. Tokio corpus groups measure complete
1,000/10,000-document batches with 1/8/64 callers, four async scheduler threads
and a bounded blocking pool. Each synchronous native call is separately
submitted and awaited, never run on an async scheduler thread. Input generation,
runtime creation and complete lossless preflight are outside timing. Timed
batches include task admission, parse/projection, document drop and completions.
One iteration means one full batch; throughput counts documents. Calibration
can repeat batches, so this is not an exact-N receipt. Standard Criterion CLI
options select filters and baselines. Profiled builds are rejected.

Fixed-count semantic/lifecycle qualification is separate:
`just qualify runtime-rust OUTPUT` invokes the Cargo example for 1,000/10,000
documents, 64 callers and 1/4/8 native domains. Explicit arguments replace
ambient corpus mode variables. The `std`/`tokio` modes require one domain.
The `qualification` mode checks lossless roundtrip and full graph/tree hashes;
`performance` remains diagnostic-only, not Criterion or 100ms ledger admission.
The deterministic corpus diversifies three fixtures; it is not 10,000
independently authored documents. Complete-count, identity, artifact-parity,
worker-exit and five-second progress checks remain. Multi-process domains
belong only to this qualification experiment, not production parsing.

`cargo test --test native_host_children` checks fresh-process normal exit,
idle/active-loop SIGTERM, stdin flags, preservation of custom host handlers and
host child exit statuses after explicit startup. The supervisor never starts
the native runtime. Historical lazy-startup failures remain recorded in
`docs/native-runtime-tokio-pr7c-20261003.org`; they are not fresh receipts for
the explicit-startup implementation. Release/package and full consumer gates
remain separate from these focused checks.
See `docs/native-runtime-explicit-startup-20261003.org` for the new startup
contract, fresh receipts and explicitly unqualified boundaries.

The public API supports:

```rust
use orgize::org_aot::{org_contract_pack, parse_org_aot};

// SAFETY: first call is before application workers/children, with exclusive
// host stdio/signal ownership; the linked parser uses neither Scheme I/O nor children.
unsafe { orgize::initialize_native_runtime() }.expect("native startup");
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
Its APIs are deliberately separate: `orgizepy.parser` parses raw Org
through native Gerbil AOT and Rust/Rowan projection, `orgizepy.functions`
reads the generated headline functions, and `orgizepy.contract` evaluates
explicit Element rows through the existing PyO3/Rust extension. Parsing and Contract
evaluation share the selected native runtime inside the maturin extension;
Python does not load a second standalone Scheme runtime. The host-resource
ownership gates above also apply to Python, not only Rust.

```python
from orgizepy.parser import parse_org
from orgizepy.functions import headline_functions
from orgizepy import initialize_native_runtime

initialize_native_runtime()
document = parse_org("#+TODO: NEXT DONE\n* NEXT Ship SDK\n")
headline = next(element for element in document.elements if element.kind == "headline")
assert headline_functions(document, headline.id).todo_keyword == "NEXT"
```

The Contract ABI is also independently consumable from C or Rust. It requires
one Gambit runtime initialization per process; see
[`bindings/c/include/orgize_standalone.h`](bindings/c/include/orgize_standalone.h).

Named Element queries are declared in Orgize's `scheme :org-elements-query` blocks
and compiled into a typed Rust pack at development time. Query evaluation uses
that pack; document parsing uses the linked Gerbil program described above:

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
Headline `title`, `raw-value`, `priority`, `tags`, and `todo-keyword` queries
reuse Scheme-AOT Element properties. Exact and contains matching of TODO
keywords both honor the document's own declarations, not a global list.

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
installing Gerbil. Contract queries reuse Element `all-of`/`any-of` property
groups and file-local TODO semantics rather than defining a second matcher.
This AOT path does not make host-loaded legacy contract
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
