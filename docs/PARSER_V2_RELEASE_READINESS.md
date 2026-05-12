# Parser v2 release readiness

This document is the PR-closeout checklist for the parser v2 lane. It records
the intended public boundary, completed parser coverage, deferred work, and
validation gates for the breaking semantic AST API.

## Public boundary

- `orgize::ast` is the owned semantic AST API.
- `Org::document()` returns `ast::ParsedAst`.
- `ast::ParsedAst` is `Document<ParsedAnnotation>`.
- `ast::BareAst` is `Document<()>`.
- `Org::syntax_document()` and `orgize::syntax_ast::*` expose the rowan-backed
  lossless typed syntax wrappers. Wrapper names that would collide with the
  semantic AST use a `Syntax` prefix, for example `SyntaxDocument`,
  `SyntaxLink`, and `SyntaxTimestamp`.
- The rowan tree remains the lossless parser substrate. The semantic AST is a
  projection layer for exporters, indexers, tests, and higher-level consumers.
- Semantic traversal is Rust-style: `visit`, `visit_mut`, `fold`, `map_ann`,
  and `try_map_ann`.

Code that previously imported rowan wrappers from `orgize::ast::*` should move
those imports to `orgize::syntax_ast::*`.

## Completed coverage

- Current lossless syntax surface has semantic projection coverage tests that
  reject accidental `Unknown` fallthrough.
- Headline, planning, property drawer, list, table, drawer, block, keyword,
  footnote, inline babel, citation, link, timestamp, macro, target, radio
  target, and text-markup projection are covered.
- Inlinetasks project into semantic `ElementData::Inlinetask` with title
  objects, planning, properties, optional `END`, body elements, and traversal.
- Source/example block metadata includes line numbering, preserve indentation,
  code references, and source block header args.
- Tables expose formulas and column alignment metadata while preserving row and
  cell content.
- Per-file TODO declarations are applied before headline projection.
- Preprocessing directives are explicit side tables: `#+INCLUDE:` directives
  and `#+MACRO:` definitions are collected without changing the lossless tree.
- Macro expansion is opt-in through semantic side-table helpers.
- Internal links resolve against document-local headline, `CUSTOM_ID`, org-id
  `ID`, target, radio target, footnote, and coderef targets while preserving
  original paths.
- Quote punctuation remains plain text; text markup inside quote boundaries is
  parsed according to Org's text-markup PRE/POST rules.
- Lesser-used semantic element variants are covered for comments, drawers,
  fixed-width areas, LaTeX environments, dynamic blocks, verse/center/comment
  blocks, and named special blocks.
- Semantic traversal compatibility is covered for all annotation-bearing node
  categories exposed through `AstRef` and `AstMut`.
- Existing HTML/export traversal continues to use the lossless syntax substrate.
- Markdown export is available through the lossless syntax traversal substrate
  via `MarkdownExport` and `Org::to_markdown()`, including fenced blocks,
  Markdown snippets/export blocks, timestamps, hard line breaks, and basic
  table output. Org tables without rule rows get a Markdown delimiter row after
  the first standard row so they remain valid Markdown tables.
- LaTeX export is available through the lossless syntax traversal substrate via
  `LatexExport` and `Org::to_latex()`, including raw LaTeX fragments,
  environments, snippets, and `#+begin_export latex` blocks.
- Criterion smoke coverage includes `Org::to_markdown()` and `Org::to_latex()`
  alongside parse, semantic projection, macro expansion, radio-link projection,
  annotation projection, and HTML export paths.

## Intentional gaps

- Existing HTML/Markdown/LaTeX export is not rewritten to semantic AST in this
  PR.
- `#+INCLUDE:` expansion does not read external files; consumers can implement
  expansion using the collected directive side table.
- Macro calls are not substituted during parsing; expansion remains opt-in.
- Quote punctuation is not a dedicated semantic `Quote` object because core Org
  syntax does not define one.
- The local foreign reference checkout is research-only. This repository does
  not vendor, translate, depend on, execute, or test against foreign parser
  sources, foreign test cases, generated artifacts, or foreign toolchains.

## Validation gate

Run every release-readiness pass through the repository environment:

```sh
direnv exec . cargo fmt --all -- --check
direnv exec . cargo test --workspace --all-targets --all-features
direnv exec . cargo clippy --workspace --all-targets --all-features -- -D warnings
direnv exec . cargo test --doc --all-features
git diff --check
rg -n "<foreign-reference-and-policy-boundary-pattern>" Cargo.toml Cargo.lock src tests README.md .github wasm examples build.rs benches docs --glob '!target/**'
```

The final boundary grep should use the repository's current forbidden
reference/toolchain and policy-bypass pattern, and it should return no matches.
The local `.data/` directory may remain untracked as a research checkout, but
it must not enter Cargo, CI, the release package, tests, or generated
artifacts.

## PR closeout

- Keep PR #4 on `parser-v2.0`; do not split a new PR for this lane.
- Keep the PR ready for review once local gates and GitHub `test` pass.
- `gh-pages` may be skipped on parser-only pushes.
- Treat unresolved review threads as blocking until resolved or explicitly
  marked non-actionable.
- Do not downgrade or disable harness policy to land this lane.
- Keep the Rust project harness mounted from root `build.rs`, `wasm/build.rs`,
  and the `src/lib.rs` cargo-test gate so both workspace packages have
  filter-proof build-time enforcement.
