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
- [X] Org `Org::to_org`
- [ ] LaTeX

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
  - [x] Build a document-local target table for headlines, custom IDs, targets,
        footnotes, radio targets, and coderefs.
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
