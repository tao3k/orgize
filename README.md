# Orgize

[![Crates.io](https://img.shields.io/crates/v/orgize.svg)](https://crates.io/crates/orgize)
[![Documentation](https://docs.rs/orgize/badge.svg)](https://docs.rs/orgize)
[![Build status](https://img.shields.io/github/actions/workflow/status/tao3k/orgize/ci.yml)](https://github.com/tao3k/orgize/actions/workflows/ci.yml)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

A Rust library for parsing org-mode files.

Live Demo: <https://tao3k.github.io/orgize/>

## Parse

To parse an org-mode string, call `Org::parse` and then project the owned
semantic AST with `Org::document()`:

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

Use `ParseConfig::parse` to specify a custom parse config. Low-level typed
syntax wrappers live under `orgize::syntax_ast`:

```rust
use orgize::{syntax_ast::Headline, Org, ParseConfig};

let config = ParseConfig {
    // custom todo keywords
    todo_keywords: (vec!["TASK".to_string()], vec![]),
    ..Default::default()
};
let org = config.parse("* TASK Title 1");
let hdl = org.first_node::<Headline>().unwrap();
assert_eq!(hdl.todo_keyword().unwrap(), "TASK");
```

`Org::document()` returns `ast::ParsedAst`, an owned semantic tree with source
annotations and projection diagnostics. Use `document.to_bare()` when tests,
snapshots, or serialization do not need source ranges.

Semantic timestamps include parsed metadata for date/time start, range end,
repeater, and warning cookies, while retaining the original raw timestamp text.
Semantic links include owned path/target data, parsed description objects,
caption metadata, and image-link detection.
Radio links keep the lightweight plain-text projection by default. Set
`radio_link_projection: RadioLinkProjection::Semantic` in `ParseConfig` to run
the opt-in semantic pass that can link parsed object spans such as
`<<<*Radio*>>>` against `*Radio*`.
Semantic source/example blocks include parsed line-numbering metadata for
`-n` and `+n` switches, optional starting offsets, preserve-indentation
metadata for `-i`, code-reference cookies from the default `(ref:name)` format
or custom `-l` label formats, line-level source/value/normalized-value records,
typed `-k`/`-r`/`-l` switch metadata, and structured source block header
arguments from `#+PROPERTY: header-args`, `#+PROPERTY: header-args:LANG`,
`#+HEADER:`, and the `#+BEGIN_SRC` line while retaining the raw parameter text.
Tangle metadata includes non-executing `:mkdirp`, `:comments`, `:shebang`, and
`:noweb` planning flags alongside the target file mode.
Result planning metadata normalizes `:results` collection, format, insertion,
value/output, and `:file` output hints without executing source blocks.
Execution/export planning metadata normalizes `:eval`, `:exports`, `:cache`,
`:session`, `:dir`, `:hlines`, and context-specific `:noweb` behavior as
source-grounded policy hints without running code or touching the filesystem.
`source_block_references()` projects non-executing literate-programming edges for `#+CALL`,
`call_name(...)`, source-block `:var` dependencies, and noweb `<<name>>`
references, resolving them against local `#+NAME` and syntax-appropriate
`:noweb-ref` declarations. Fixed-width areas use the same semantic
line record shape, and inline Babel source/call contexts now keep nested
bracket, brace, and parenthesis bodies balanced.
Semantic tables expose column alignment metadata from `<l>`, `<c>`, and `<r>`
property cookies while preserving the original row and cell contents.
Per-file TODO declarations from `#+TODO:`, `#+SEQ_TODO:`, and `#+TYP_TODO:`
are applied before parsing headlines, so custom TODO/DONE states are projected
into semantic headline metadata for that document.
Inlinetasks use `ParseConfig::inlinetask_min_level`, defaulting to Org's level
15 convention, and project as semantic elements with parsed title objects,
planning, properties, optional `END` markers, and body elements.
Quote punctuation is not modeled as a semantic object; it remains plain text,
while normal objects inside quote punctuation still project independently.
Preprocessing directives stay explicit in parser v2. `#+INCLUDE:` and
`#+MACRO:` remain normal keyword elements in the lossless tree, and the
semantic document also collects include directives and macro definitions into
document-level side tables. Macro calls are parsed as objects without expanding
their templates by default. Use `document.macro_expansions()` for opt-in macro
substitution side tables when an exporter or indexer wants expanded macro text.
Document-local link targets are collected into `document.targets`, covering
headlines, `CUSTOM_ID` and org-id `ID` properties, explicit targets, radio
targets, footnote definitions, and source/example block coderefs. Link
projection resolves these targets into `LinkTarget::Internal` while keeping the
original `link.path()`, and reports diagnostics for ambiguous or missing strict
internal links.
Parser v2 also collects exporter/indexer side tables without mutating the
lossless source tree: `document.metadata`, `document.filetags`,
`document.tag_definitions`, `document.export_settings`, `document.link_abbreviations`, and
`document.footnotes`. Keyword values keep their raw source text and expose
parsed object values for metadata-style keywords such as `#+TITLE:` and
`#+CAPTION:`; `ATTR_*` affiliated keywords expose shell-like structured
attributes. Links without an explicit description can use target-derived
fallback objects through `Link::description_or_default()`, and `id:ID::*search`
paths retain their search suffix in `Link::search`.
`#+TAGS:` vocabulary lines project shortcuts such as `EMACS (e)` and `READ(r)`
into `document.tag_definitions`, including official `{ ... }` mutually
exclusive sets and `[ group : members ]` hierarchy metadata. Hosts that want
org-mode-style programmable behavior can call `document.org_elements_json()` or
explicitly run
`document.execute_org_elements(&OrgElementsHostExecutionOptions::new(...))`;
agenda, sparse-tree, workspace match, and clocktable `:match` projections
expand group tags from that vocabulary without mutating headline tags,
including `TAGS` and `ALLTAGS` special-property match expressions.
the payload exposes source-backed root/section/element/object trees, targets,
footnotes, metadata, source block side tables, and a flat `index` for
`org-element-map`-style filtering by node kind. Rust consumers can call
`document.org_elements_index()` for typed index records and
`document.query_org_elements_index(&OrgElementsIndexQuery::new().kind("link"))`
for filtered views; the query can also match compact summary fields with
`summary_eq` and `summary_contains` selectors. Wasm consumers can request
`orgElementsIndexJson()` or `orgElementsIndexQueryJson(...)` without
materializing the full tree.
Parsing alone never executes host tools or header directives. Python remains a
convenience adapter through `PythonExecutionOptions`, not the core element
contract.
Org Crypt is modeled as source-grounded advice, not an execution feature:
`document.crypt_states()` records `crypt`-tagged sections, inherited crypt tag
evidence, `CRYPTKEY` visibility, encrypted payload markers, and opaque-body
warnings while never decrypting source text. The wasm package exposes the same
shape through `cryptJson()` and `snapshotJson().crypt`.
Runtime-adjacent Org features stay source-grounded too:
`document.runtime_metadata_plan()` records FEEDSTATUS drawers, relative timer
stamps, MobileOrg index and `FLAGGED`/`ORIGINAL_ID` metadata, plus explicit
feed/timer/mobile/persist execution boundaries without network access,
filesystem sync, or cache writes. Wasm consumers can request
`runtimeMetadataJson()` or read `snapshotJson().runtimeMetadata`.
Use `document.link_protocol_records()` to inspect built-in link families,
custom protocols, `#+LINK` abbreviations, executable `shell:`/`elisp:` links,
and inert `org-protocol:` calls without opening files or dispatching handlers.
Use `document.column_summary_plans()` to inspect non-mutating Column View
summary behavior for `COLUMNS` declarations, including common Org operators
such as `+`, `:`, `X%`, and `est+`.
Use `document.project_for_export(&ExportProjectionOptions::default())` as the
opt-in semantic projection hook for exporter-oriented pruning and transformations
such as `COMMENT`/`:ARCHIVE:`/tag pruning, link abbreviation expansion, and
special-string conversion. `Org::to_html()`, `Org::to_markdown()`, and
`Org::to_latex()` keep their existing default output stable; the corresponding
`*_with_options` methods expose opt-in special-string and entity handling.
Use `document.agenda_entries(&AgendaQuery::new(start, end))` when an indexer or
UI wants an Org Agenda-style semantic view over planning and plain active
timestamps. The projection derives scheduled, deadline, warning, overdue,
closed, active timestamp, repeated, and tag-filtered rows, including timestamp
range display days and start/end times, headline time-of-day ranges, scheduled
delay cookies, and `CATEGORY` keyword/property metadata without changing the
parsed document or exporter defaults. Use
`document.agent_planning_snapshot(&AgentPlanningQuery::new(query))` when an
agent wants compact decision cards over those agenda rows. The snapshot is a
renderer-friendly projection of official Org agenda semantics; its `PLANxxx`
codes are output diagnostics, not Org source syntax.
Use `AgendaWorkspaceBuilder` with `AgendaWorkspaceQuery` when a caller already
has multiple parsed documents and wants built-in Agenda-style command plans for
daily agenda rows, TODO lists, tag/property matches, text search, and stuck
projects without letting orgize scan agenda files itself. Agenda view cards now
carry `AgendaUrgencyScore` ingredients for explainable ranking. Use
`document.citation_export_plan()` for Org Cite bibliography/processor/print
bibliography side tables, `agent_capture_plan(&AgentCaptureRequest::new(...))`
for non-mutating Agent capture previews over native Org entries,
`publishing_project_plan()` for explicit blog/site publishing graphs,
`export_dependency_graph()` for combined include/setupfile/bibliography/macro
and publishing-output dependency graphs, `document.table_visualization_plans()`
for non-executing Org Plot/radio-table metadata, and
`document.attachment_inventory()` for opt-in filesystem attachment evidence.

Use `Org::syntax_document()` when you need the lossless rowan-backed syntax tree:

```rust
use orgize::{rowan::ast::AstNode, syntax_ast::Headline, Org};

let org = Org::parse("* Title");
let syntax_doc = org.syntax_document();
let headline = syntax_doc.syntax().children().find_map(Headline::cast).unwrap();
assert_eq!(headline.title_raw(), "Title");
```

## Traverse

Use `org.traverse(&mut traversal)` to walk through the syntax tree.

```rust
use orgize::{
    export::{from_fn, Container, Event},
    Org,
};

let mut hdl_count = 0;
let mut handler = from_fn(|event| {
    if matches!(event, Event::Enter(Container::Headline(_))) {
        hdl_count += 1;
    }
});
Org::parse("* 1\n** 2\n*** 3\n****4").traverse(&mut handler);
assert_eq!(hdl_count, 3);
```

## Modify

Use `org.replace_range(TextRange::new(start, end), "new_text")` to modify the syntax tree:

```rust
use orgize::{syntax_ast::Headline, Org, TextRange};

let mut org = Org::parse("hello\n* world");

let hdl = org.first_node::<Headline>().unwrap();
org.replace_range(hdl.text_range(), "** WORLD!");

let hdl = org.first_node::<Headline>().unwrap();
assert_eq!(hdl.level(), 2);

org.replace_range(TextRange::up_to(hdl.start()), "");
assert_eq!(org.to_org(), "** WORLD!");
```

## Render to html

Call the `Org::to_html` function to export org element tree to html:

```rust
use orgize::Org;

assert_eq!(
    Org::parse("* title\n*section*").to_html(),
    "<main><h1>title</h1><section><p><b>section</b></p></section></main>"
);
```

Checkout `examples/html-slugify.rs` on how to customizing html export process.

## Render to Markdown

Call the `Org::to_markdown` function to export the org element tree to
Markdown:

```rust
use orgize::Org;

assert_eq!(
    Org::parse("* title\n*section*").to_markdown(),
    "# title\n**section**\n"
);
```

## Render to LaTeX

Call the `Org::to_latex` function to export the org element tree to LaTeX body
text:

```rust
use orgize::Org;

assert_eq!(
    Org::parse("* title\n*section* and $a_b$").to_latex(),
    "\\section{title}\n\\textbf{section} and $a_b$\n\n"
);
```

## Command line

The `orgize` binary exposes parser-backed `lint`, conservative `fmt`, and
read-only SDD status commands:

```sh
orgize lint notes.org
orgize lint --format text notes.org
orgize lint --json notes.org
orgize lint --priority-highest 0 --priority-default 5 --priority-lowest 9 notes.org
orgize fmt --check notes.org
orgize fmt notes.org docs/
orgize sdd status notes.org
```

`lint` reports semantic projection diagnostics, document-local target
uniqueness issues such as duplicate `ID`/`CUSTOM_ID` targets, missing local
macro definitions, duplicate `#+MACRO:` definitions, malformed or duplicate
`#+LINK:` abbreviation definitions, invalid supported `#+OPTIONS:` values,
duplicate or conflicting per-file TODO keyword declarations, and missing or
non-file `#+INCLUDE:` paths when linting real files. Priority-cookie checks use
Org's default `A..C` profile unless callers pass
`--priority-highest/--priority-default/--priority-lowest`, which also supports
numeric profiles such as `0..9`. By default, `lint` prints a
compact agent-facing repair report with location, source line, fix hint, and
contract; `--format text` keeps the stable line-oriented form, and `--json`
keeps structured machine output as an explicit mode. `fmt` starts with
source-safe whitespace normalization: it trims trailing spaces and tabs, aligns
contiguous Org tables outside blocks, normalizes final blank lines, and ensures
one final newline for non-empty documents. When paths are provided, `fmt`
writes files by default; with no path it reads stdin and writes stdout. Both
commands accept multiple file and directory paths; directory paths are expanded
recursively to `.org` files, and explicit file operands must be `.org` files.
Formatter behavior is covered by snapshot tests so future formatting expansions
review as explicit output diffs.
`sdd status` projects Org-native SDD headings from ordinary tags and property
drawers without writing files. It treats `ID` as the stable machine identity,
`SDD_PARENT` as a semantic Org `id:` link edge, and reports compact
architecture/audit cards for Agent design-review surfaces. SDD headings describe
system boundaries, capabilities, views, decisions, and audits; implementation
checklists belong in linked Org task or ExecPlan files.

## Features

- **`chrono`**: adds the ability to convert `Timestamp` into `chrono::NaiveDateTime`, disabled by default.

- **`indexmap`**: adds the ability to convert `PropertyDrawer` properties into `IndexMap`, disabled by default.

## Development

Parser v2 mounts `rust-lang-project-harness` from root `build.rs` and the
`src/lib.rs` cargo-test gate. The wasm package is a standalone
`tao3k/orgize-wasm` repository mounted at `wasm/` as a git submodule; its own
`wasm/build.rs` keeps the same build-time harness policy inside that repository.
The build-time gates prevent filtered cargo test runs from bypassing blocking
project policy, while the test gate keeps compact agent advice visible during
normal local validation. All gates use the current standalone harness repository
instead of the retired monorepo-local `xiuxian-testing` crate. No rule pack or
rule severity is downgraded:
`RUST-MOD-*` and project layout findings stay blocking. `AGENT-*` `info`
findings remain visible as repair advice while this legacy crate burns them down
separately. New tests should still use explicit imports: `RUST-MOD-R010`
reports parent-scope glob imports.
The build-time gate ignores generated environment/data roots such as `.devenv/`
and `.data/` so research checkouts stay outside Cargo, CI, and published
package boundaries.

Fresh checkouts that need the browser demo or npm package should initialize the
submodule first:

```sh
git submodule update --init --recursive wasm
just wasm-build
```

## API compatibility

Parser v2 makes a breaking API boundary explicit:

- `orgize::ast` is the owned semantic AST. Its primary public types are
  `Document<A>`, `Section<A>`, `Element<A>`, `Object<A>`, `ParsedAnnotation`,
  and `Diagnostic`.
- `ast::ParsedAst` is `Document<ParsedAnnotation>`.
- `ast::BareAst` is `Document<()>`.
- `Org::document()` returns `ast::ParsedAst`.
- `Org::syntax_document()` and `orgize::syntax_ast::*` expose the old typed
  wrappers around the lossless rowan syntax tree. Wrapper names that would
  collide with semantic AST types use a `Syntax` prefix, for example
  `syntax_ast::SyntaxDocument` and `syntax_ast::SyntaxLink`.
- `Document<A>::project_for_export(&ExportProjectionOptions)` returns an
  exporter-oriented semantic projection without changing the parsed AST.
- `Document<A>::agenda_entries(&AgendaQuery)` returns an opt-in agenda
  projection over semantic planning and active timestamps without mutating
  `ParsedAst`.

Code that previously imported rowan-backed wrappers from `orgize::ast::*`
should import them from `orgize::syntax_ast::*` instead.

`Org::syntax_document()` and the `syntax_ast` module expose access to the internal syntax tree,
along with rowan low-level APIs. This can be useful for intricate tasks or for
HTML/export integrations that need byte-for-byte source preservation.

However, the structure of the internal syntax tree can change between different versions of the library.
Because of this, the result of `element.syntax()` doesn't follow semantic versioning,
which means updates might break your code if it relies on this method.

Semantic AST traversal is exposed through Rust-style APIs on `Document<A>`:
`visit`, `visit_mut`, `fold`, `map_ann`, and `try_map_ann`. The semantic layer is
projected from the rowan substrate; it does not replace rowan as the lossless
parser representation.

See [docs/20_parser/20.01_parser_v2_release_readiness.org](docs/20_parser/20.01_parser_v2_release_readiness.org)
for the parser v2 closeout checklist, intentional gaps, and validation gate.
