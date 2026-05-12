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
or custom `-l` label formats, and structured source block header arguments
while retaining the raw parameter text.
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
`document.export_settings`, `document.link_abbreviations`, and
`document.footnotes`. Keyword values keep their raw source text and expose
parsed object values for metadata-style keywords such as `#+TITLE:` and
`#+CAPTION:`; `ATTR_*` affiliated keywords expose shell-like structured
attributes. Links without an explicit description can use target-derived
fallback objects through `Link::description_or_default()`, and `id:ID::*search`
paths retain their search suffix in `Link::search`.
Use `document.project_for_export(&ExportProjectionOptions::default())` as the
opt-in semantic projection hook for exporter-oriented pruning and transformations
such as `COMMENT`/`:ARCHIVE:`/tag pruning, link abbreviation expansion, and
special-string conversion. `Org::to_html()`, `Org::to_markdown()`, and
`Org::to_latex()` keep their existing default output stable; the corresponding
`*_with_options` methods expose opt-in special-string and entity handling.

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

The `orgize` binary exposes parser-backed `lint` and conservative `fmt`
commands:

```sh
orgize lint --format text notes.org
orgize lint --json notes.org
orgize fmt --check notes.org
orgize fmt notes.org docs/
```

`lint` reports semantic projection diagnostics and document-local target
uniqueness issues such as duplicate `ID`/`CUSTOM_ID` targets. `fmt` starts with
source-safe whitespace normalization: it trims trailing spaces and tabs,
aligns contiguous Org tables outside blocks, normalizes final blank lines, and
ensures one final newline for non-empty documents. When paths are provided,
`fmt` writes files by default; with no path it reads stdin and writes stdout.
Both commands accept multiple file and directory paths; directory paths are
expanded recursively to `.org` files. Formatter behavior is covered by snapshot
tests so future formatting expansions review as explicit output diffs.

## Features

- **`chrono`**: adds the ability to convert `Timestamp` into `chrono::NaiveDateTime`, disabled by default.

- **`indexmap`**: adds the ability to convert `PropertyDrawer` properties into `IndexMap`, disabled by default.

## Development

Parser v2 mounts `rust-lang-project-harness` from root `build.rs`,
`wasm/build.rs`, and the `src/lib.rs` cargo-test gate. The build-time gates
prevent filtered cargo test runs from bypassing blocking project policy in both
workspace packages, while the test gate keeps compact agent advice visible
during normal local validation. All gates use the current standalone harness
repository instead of the retired monorepo-local `xiuxian-testing` crate. No
rule pack or rule severity is downgraded:
`RUST-MOD-*` and project layout findings stay blocking. `AGENT-*` `info`
findings remain visible as repair advice while this legacy crate burns them down
separately. New tests should still use explicit imports: `RUST-MOD-R010`
reports parent-scope glob imports.
The build-time gate ignores generated environment/data roots such as `.devenv/`
and `.data/` so research checkouts stay outside Cargo, CI, and published
package boundaries.

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

See [docs/PARSER_V2_RELEASE_READINESS.md](docs/PARSER_V2_RELEASE_READINESS.md)
for the parser v2 closeout checklist, intentional gaps, and validation gate.
