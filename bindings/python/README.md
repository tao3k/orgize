# orgizepy

`orgizepy` is Orgize's production Python SDK. It exposes three separate APIs:

- `orgizepy.parser.parse_org` parses raw Org text using the Scheme-declared,
  Rust/Rowan AOT parser. It returns typed Elements, including all projected fields.
- `orgizepy.functions.headline_functions` reads the Scheme-AOT headline functions
  from a parsed document.
- `orgizepy.edits` computes an exact-source digest and asks the Rust/Scheme-AOT
  path to validate source-bound edits. It returns candidate Org text only;
  authorization, review, source recheck, and persistence remain with the caller.
- `orgizepy.contract.evaluate_contract` calls the independent Scheme/Gambit
  Contract ABI over explicitly admitted Element rows.

The parser and functions do not initialize Gerbil. The Contract runtime starts
once per process on the main thread and shuts down on process exit; Gambit
cannot be restarted in the same process after shutdown. Contract evaluation does not
silently flatten a parsed Element's multiple fields into the narrower Contract
ABI row shape. The standalone Contract library is bundled in the CI-built
platform wheels; source-tree tests can set `ORGIZE_CONTRACT_LIBRARY` to its
built path. A wheel rebuilt from the source distribution now fails explicitly
unless the Scheme Contract library is built first; it must not silently become
a parser-only wheel. In a repository checkout, prepare the library with
`just python-contract-library` before building a wheel; that recipe requires an
installed Gerbil toolchain and the compiled Orgize Scheme package. A standalone
sdist has its Python project at the extracted archive root. With its pinned
Gerbil dependencies installed in an isolated `GERBIL_PATH`, run
`gxi build.ss compile`, then build the Contract library into
`src/orgizepy/lib/liborgize.so` on Linux or `liborgize.dylib` on macOS using
`bindings/c/build-native-library.py --output <path>`. Finally build and repair
the wheel from that root. A complete sdist-built wheel must pass the same
installed SDK smoke test as a repository-built wheel.

Raw wheels retain build-host dynamic library paths. The `python-wheel-repair`
recipe relinks and bundles non-system dependencies; CI then tests the repaired
wheel after installation. On macOS the repair step also recalculates the
minimum supported OS tag from the bundled libraries.

The project is managed by `uv`; Rust bindings are built with PyO3 and maturin.
Bundled native and Rust dependencies are inventoried in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md); the matching license texts
are included in each wheel and source distribution.
