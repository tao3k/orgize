# Orgize implementation status

Check out https://orgmode.org/worg/dev/org-syntax.html for more information.

- [x] Headline
  - [X] Objects insides headline title
- [x] Affiliated Keywords

## Greater Elements
- [x] Greater Blocks
- [X] Drawers and Property Drawers
- [x] Dynamic Blocks
- [x] Footnote Definitions
- [x] Inlinetasks
  - [x] Objects inside inlinetask title
- [x] Plain Lists and Items
  - [x] Nested List
  - [x] Nested List Indentation
  - [x] Tag
  - [x] Counter
  - [x] Counter set
- [X] Property Drawers
- [X] Tables

## Elements

- [x] Babel Call
- [x] Blocks
  - [x] Escape characters (`#`,`*`, etc)
  - [x] Line numbers
- [X] Clock, Diary Sexp and Planning
- [x] Comments
- [x] Fixed Width Areas
- [x] Horizontal Rules
- [x] Keywords
- [x] LaTeX Environments
- [X] Node Properties
- [x] Paragraphs
- [X] Table Rows

## Objects

- [x] Entities and LaTeX Fragments
- [x] Export Snippets
- [x] Footnote References
- [x] Inline Babel Calls and Source Blocks
- [x] Line Breaks
- [x] Links
  - [x] Regular link
  - [x] Angle link
  - [x] Plain link
  - [x] Radio link (semantic AST projection)
- [x] Macros
- [x] Targets and Radio Targets
- [x] Statistics Cookies
- [x] Subscript and Superscript
- [X] Table Cells
- [x] Timestamps
- [x] Text Markup
  - [x] bold
  - [x] italic
  - [x] underline
  - [x] verbatim
  - [x] code
  - [x] strike-through

## Export

- [x] HTML `Org::to_html`
- [x] Markdown `Org::to_markdown`
- [X] Org `Org::to_org`
- [x] LaTeX `Org::to_latex`

## Extra

- [X] Syntax Highlighting

## Parser v2 Alignment Backlog

This checklist tracks Org features that are not fully covered by the current
lossless parser plus semantic AST projection. It is derived from a local
research pass over org-element-like parser classifications and exporter-side
Org behavior. The reference checkout remains research-only: no foreign source,
tests, generated output, build tooling, or runtime dependency is imported into
this repository.

### M3 Parser Gaps

- [x] Inlinetasks
  - [x] Parse inlinetask headlines and bodies.
  - [x] Parse objects inside inlinetask titles.
- [x] Source/example block coderefs
  - [x] Preserve per-line coderef cookies such as `(ref:name)`.
  - [x] Respect custom `-l` coderef label formats.
  - [x] Expose coderef metadata in semantic block lines without losing raw
        block text.
- [x] Source/example block indentation semantics
  - [x] Model `-i` preserve-indentation behavior separately from raw source
        retention.
  - [x] Keep tab-width-sensitive normalization as an explicit exporter/indexer
        choice.
- [x] Source block header arguments
  - [x] Project raw `:key value` parameters into structured semantic header
        args.
  - [x] Keep raw parameter text for round-trip and compatibility.
- [x] Table column metadata
  - [x] Detect column property rows such as `<l>`, `<c>`, and `<r>`.
  - [x] Expose semantic column alignment metadata while preserving current
        row/cell text.
- [x] Quoted objects
  - [x] Keep single/double quote punctuation as plain text because core Org
        syntax does not define a dedicated quote object.
  - [x] Verify objects inside quote punctuation continue to project normally.

### Pre/Post Processing Gaps

- [x] Include keyword expansion
  - [x] Parse `#+INCLUDE:` as a normal keyword without expansion.
  - [x] Add an explicit expansion hook or side-table design before resolving
        external files.
- [x] Per-file TODO keyword declarations
  - [x] Parse `#+TODO:`, `#+SEQ_TODO:`, and `#+TYP_TODO:` declarations.
  - [x] Apply declarations before headline semantic projection, or expose a
        two-pass projection API.
- [x] Macro definitions and substitution
  - [x] Collect `#+MACRO:` definitions.
  - [x] Keep macro calls parsed even when substitution is disabled.
  - [x] Add opt-in expansion semantics without changing the lossless tree.
- [x] Internal link resolution
  - [x] Build a document-local target table for headlines, custom IDs, org-id
        IDs, targets, footnotes, radio targets, and coderefs.
  - [x] Resolve `LinkTarget::Unresolved` where possible while keeping the
        original link path.
  - [x] Preserve diagnostics for ambiguous or missing targets.
- [x] Full radio-link conformance
  - [x] Current semantic projection links plain text against collected radio
        targets.
  - [x] Add opt-in two-pass behavior for edge cases that require preprocessing
        before object parsing.

### M4 Traversal/Export Compatibility

- [x] Keep the current HTML/export pipeline on the lossless syntax substrate.
- [x] Cover semantic traversal shapes for exporter/indexer consumers.
- [x] Verify `visit`, `visit_mut`, and `fold` reach all annotation-bearing
      semantic node categories, including preprocessing and metadata nodes.

### M5 PR Closure And Release Readiness

- [x] Document breaking parser v2 API boundaries.
- [x] Document intentional gaps and deferred exporter work.
- [x] Document the full local validation gate.
- [x] Document the no-Haskell-dependency research boundary.
- [x] Align the wasm demo with parser v2 feature coverage and expose semantic
      AST output in the browser demo.

### M6 Semantic Projection Performance

- [x] Reuse the document-local target index as the radio-link projection source.
- [x] Avoid a second semantic pre-scan for radio target collection.
- [x] Merge target indexing and preprocessing directive collection into one
      semantic pre-scan.
- [x] Scan each radio-link object run once and prefer the longest target at a
      shared start offset.
- [x] Precompute object-run spans so radio-link prefix/description/suffix
      slicing does not rescan the whole run from the first object.
- [x] Return unmatched plain-text radio-link objects without cloning their
      annotations or reallocating their text.
- [x] Use capacity-aware macro definition lookup and reuse `$0` argument joins
      within one macro expansion.
- [x] Add a dense macro-expansion benchmark.
- [x] Add a dense target/radio-link projection benchmark.
- [x] Use hash-based target lookup and move collected target definitions into
      the final semantic document without cloning the target table.
- [x] Avoid allocating normalized target keys for link lookups that miss the
      document-local target index.
- [x] Use O(1) source-column projection for ASCII lines while preserving
      character-accurate UTF-8 columns through per-line char indexes.
- [x] Add a dense annotation projection benchmark.
- [x] Append plain-text radio-link projection directly into the output object
      buffer instead of allocating a temporary vector per source object.
- [x] Preallocate radio-link projection buffers from input object counts.
- [x] Scan block begin/content children once when projecting semantic block
      metadata, value, and nested elements.

### M7 LaTeX Export

- [x] Add a lossless traversal based `LatexExport` handler.
- [x] Add `Org::to_latex()` as the public convenience API.
- [x] Preserve raw LaTeX fragments, LaTeX environments, LaTeX export blocks,
      and LaTeX snippets.
- [x] Cover headline, paragraph, markup, block, list, table, link, timestamp,
      citation, entity, and subtree rendering behavior with integration tests.
- [x] Expose LaTeX output in the wasm parser demo.

### M8 Status TODO Burn-down

- [x] Remove the stale `Link::caption()` paragraph-only limitation by resolving
      the nearest ancestor caption keyword.
- [x] Cover caption lookup through a captioned block ancestor in doctests.
- [x] Replace the stale `element_nodes` blank-line TODO with the current
      lossless parser invariant.
- [x] Expand block parser coverage for export, quote, and special blocks.

### M9 Markdown Export API

- [x] Add `Org::to_markdown()` as the public convenience API.
- [x] Use `to_markdown()` in the Markdown example.
- [x] Render source, example, and fixed-width content as fenced Markdown blocks.
- [x] Preserve Markdown snippets and `#+begin_export markdown` blocks.
- [x] Render timestamps, hard line breaks, and basic Org tables in Markdown.
- [x] Expose Markdown output in the wasm parser demo.

### M10 Release-Candidate Hardening

- [x] Cover semantic projection of lesser-used elements: comments, drawers,
      fixed-width areas, LaTeX environments, dynamic blocks, verse/center/comment
      blocks, and named special blocks.
- [x] Make Markdown tables without Org rule rows render a Markdown header
      delimiter instead of emitting a non-table pipe block.
- [x] Add Criterion smoke coverage for `Org::to_markdown()` and
      `Org::to_latex()` alongside parse, semantic projection, and HTML export.

### M11 Harness Build-Time Coverage

- [x] Keep root `build.rs` mounted as a filter-proof Rust project harness gate.
- [x] Add `rust-lang-project-harness` to `orgize-wasm` `[build-dependencies]`.
- [x] Mount the same build-time harness gate from `wasm/build.rs` before the
      existing wasm build metadata emission.

### M12 Harness Advice Enforcement

- [x] Enforce the default advice surface in the `src/lib.rs` cargo-test harness
      gate.
- [x] Verify the cargo-test gate now fails on current advisory findings instead
      of hiding them behind passing test output.
- [x] Close `AGENT-R004` public namespace conflicts between semantic AST and
      legacy `syntax_ast` wrappers without weakening the v2 public API boundary.
- [x] Close `AGENT-R009` syntax parser owner-cycle findings by moving shared
      parser contracts behind clearer owner boundaries.
- [x] Close semantic AST model advice for primitive/stringly public state
      (`AGENT-R020`, `AGENT-R028`) with typed domain carriers.
- [x] Split or extract the semantic projection hot paths flagged by
      `AGENT-R025`/`AGENT-R026`, including `conversion.rs` radio-link,
      footnote, and object-run helpers.

### M13 Release/Performance Guard

- [x] Verify PR #4 has no unresolved review threads before continuing the next
      parser-v2 closeout slice.
- [x] Add dense semantic radio-link projection benchmark coverage for parsed
      object runs after the parser contract split.

### M14 Org-Id Resolution

- [x] Collect headline `:ID:` properties as document-local org-id targets.
- [x] Resolve matching `id:` links while preserving the original link path and
      leaving unmatched external org-id links available as URI-like links.

### M15 Reference Parser Alignment Audit

This checklist records the remaining semantic/parser gaps found by reading the
local foreign reference parser and exporter-processing model. The checkout is
research-only: no foreign source, tests, generated output, build tooling, or
runtime dependency is copied into this repository.

- [x] Parsed keyword value model
  - [x] Preserve raw keyword text while also projecting parsed object values for
        document metadata keywords such as `#+TITLE:`, `#+AUTHOR:`, `#+DATE:`,
        and parsed affiliated keywords such as `#+CAPTION:`.
  - [x] Project `ATTR_*` affiliated keywords into backend-specific structured
        arguments instead of exposing only raw keyword value text.
- [x] Document/export setting side tables
  - [x] Collect `#+FILETAGS:` as document-level metadata and decide how file
        tags participate in section tag inheritance.
  - [x] Parse export control keywords such as `#+OPTIONS:`, `#+SELECT_TAGS:`,
        and `#+EXCLUDE_TAGS:` into a typed settings side table for exporters and
        indexers.
  - [x] Model exporter-facing toggles for special strings, entity expansion,
        headline export depth, and headline level shifting without changing the
        lossless syntax substrate.
- [x] Export pruning semantics
  - [x] Add an opt-in semantic/export pass for `COMMENT` headlines,
        `:ARCHIVE:`, selected tags, and excluded tags.
  - [x] Keep pruning out of the parser itself so `ParsedAst` remains a faithful
        source projection.
- [x] Link post-processing parity
  - [x] Generate stable unique headline anchors for headlines without
        `CUSTOM_ID` or `ID`, including collision handling.
  - [x] Use target aliases as default descriptions for links without explicit
        descriptions, including headline titles, explicit targets, radio
        targets, and source-backed target kinds. List item counters remain
        semantic list metadata rather than a linkable target source in the
        current lossless syntax surface.
  - [x] Support configurable link abbreviations before final URI/internal link
        classification.
  - [x] Extend `id:ID::*search` handling from local ID normalization to search
        context resolution when an exporter/indexer requests it.
- [x] Footnote post-processing parity
  - [x] Register inline footnote definitions in a document side table.
  - [x] Assign deterministic generated labels for anonymous inline footnotes.
  - [x] Resolve footnote references against both standalone definitions and
        inline definitions without mutating the lossless tree.
- [x] Citation grammar parity
  - [x] Parse citation bodies with balanced bracket handling instead of stopping
        at the first `]`.
  - [x] Align org-cite key parsing and global/reference prefix/suffix parsing
        with the narrower citation object context.
  - [x] Add semantic diagnostics for malformed citation segments that still
        reach the lossless syntax tree.
- [x] Object-context parity
  - [x] Decide whether single/double quoted spans should become dedicated
        semantic quote objects or remain the current intentional plain-text
        punctuation behavior.
  - [x] Audit minimal-vs-standard object parsing contexts for citations,
        subscript/superscript bodies, list tags, and footnote inline
        definitions so nested objects are neither under-parsed nor over-parsed.
- [x] Export-only text transformations
  - [x] Add opt-in exporter transformations for special strings such as `--`,
        `---`, `...`, escaped hyphen, and apostrophe replacement.
  - [x] Keep entity expansion configurable per exporter while semantic AST keeps
        the source-backed entity object.

### M16 M15 Projection Performance Guard

- [x] Verify PR #4 has no unresolved review threads and the latest GitHub CI
      `test` check is green before adding the next parser-v2 slice.
- [x] Cover HTML/Markdown/LaTeX exporter options for opt-in special strings and
      entity preservation.
- [x] Add dense M15 benchmark coverage for metadata/settings, link
      abbreviations, target alias/default descriptions, citations, inline
      footnotes, and export projection pruning.
- [x] Precompute export select/exclude tag sets once per projection instead of
      rebuilding lowercase hash sets for every section.

### M17 Parser v2 Closeout Evidence

- [x] Add `docs/PARSER_V2_PERFORMANCE_CLOSEOUT.md` as the durable performance
      closeout entry point.
- [x] Record the parser-v2 hot-path ownership matrix, dense benchmark matrix,
      and current low-sample Criterion measurements.
- [x] Link release readiness to the structured performance closeout document.

### M18 Orgize lint/fmt CLI

- [x] Add a thin `orgize` binary entrypoint with implementation kept in the
      library-owned CLI module for harness compliance.
- [x] Add `orgize::lint` for semantic diagnostics and duplicate document-local
      target checks, including duplicate `ID`/`CUSTOM_ID` target errors.
- [x] Add `orgize::fmt` with conservative source-safe whitespace formatting.
- [x] Snapshot the formatter output contract, `fmt --check` CLI output, and
      lint text/JSON output.
- [x] Make `orgize fmt PATH...` write files by default, while preserving
      stdin-to-stdout behavior when no path is provided.
- [x] Expand directory paths recursively to `.org` files and support multiple
      file/path operands.
- [x] Add Org table alignment outside block bodies, covered by formatter
      snapshots.
- [x] Reuse directory and multi-file path expansion for `orgize lint`.
- [x] Snapshot `fmt --check` directory output and stdin table formatting.
- [x] Snapshot indented table alignment, pipe-only rule rows, formula
      preservation, stdin `--check`, and formatter idempotence.
- [x] Check collected `#+INCLUDE:` directives for missing or non-file local
      paths when linting real files, while keeping include expansion out of the
      parser and semantic AST.
- [x] Warn on semantic macro calls that have no matching local `#+MACRO:`
      definition, using the existing opt-in macro expansion side table instead
      of mutating parser output.
- [x] Warn on duplicate `#+MACRO:` definitions before opt-in macro expansion
      chooses a local template.
- [x] Warn on malformed or duplicate `#+LINK:` abbreviation definitions without
      treating unknown URI schemes as broken abbreviation uses.
- [x] Warn on invalid values for the supported `#+OPTIONS:` parser-v2 export
      settings (`H`, `-`, and `e`) while ignoring unknown Org option keys.
