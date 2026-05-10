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
- [ ] Inlinetasks
  - [ ] Objects insides inlinetask title
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

- [ ] Inlinetasks
  - [ ] Parse inlinetask headlines and bodies.
  - [ ] Parse objects inside inlinetask titles.
- [ ] Source/example block coderefs
  - [ ] Preserve per-line coderef cookies such as `(ref:name)`.
  - [ ] Respect custom `-l` coderef label formats.
  - [ ] Expose coderef metadata in semantic block lines without losing raw
        block text.
- [ ] Source/example block indentation semantics
  - [ ] Model `-i` preserve-indentation behavior separately from raw source
        retention.
  - [ ] Keep tab-width-sensitive normalization as an explicit exporter/indexer
        choice.
- [ ] Source block header arguments
  - [ ] Project raw `:key value` parameters into structured semantic header
        args.
  - [ ] Keep raw parameter text for round-trip and compatibility.
- [ ] Table column metadata
  - [ ] Detect column property rows such as `<l>`, `<c>`, and `<r>`.
  - [ ] Expose semantic column alignment metadata while preserving current
        row/cell text.
- [ ] Quoted objects
  - [ ] Decide whether single/double quoted objects belong in core Org syntax
        coverage or remain exporter-level typography.
  - [ ] If modeled, add semantic `Quote` object with nested children.

### Pre/Post Processing Gaps

- [ ] Include keyword expansion
  - [ ] Parse `#+INCLUDE:` as a normal keyword without expansion.
  - [ ] Add an explicit expansion hook or side-table design before resolving
        external files.
- [ ] Per-file TODO keyword declarations
  - [ ] Parse `#+TODO:`, `#+SEQ_TODO:`, and `#+TYP_TODO:` declarations.
  - [ ] Apply declarations before headline semantic projection, or expose a
        two-pass projection API.
- [ ] Macro definitions and substitution
  - [ ] Collect `#+MACRO:` definitions.
  - [ ] Keep macro calls parsed even when substitution is disabled.
  - [ ] Add opt-in expansion semantics without changing the lossless tree.
- [ ] Internal link resolution
  - [ ] Build a document-local target table for headlines, custom IDs, targets,
        footnotes, radio targets, and coderefs.
  - [ ] Resolve `LinkTarget::Unresolved` where possible while keeping the
        original link path.
  - [ ] Preserve diagnostics for ambiguous or missing targets.
- [ ] Full radio-link conformance
  - [ ] Current semantic projection links plain text against collected radio
        targets.
  - [ ] Add opt-in two-pass behavior for edge cases that require preprocessing
        before object parsing.
