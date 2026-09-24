# orgizepy

`orgizepy` is Orgize's production Python SDK. It exposes three separate APIs:

- `orgizepy.parser.parse_org` parses raw Org text using the Scheme-declared,
  Rust/Rowan AOT parser. It returns typed Elements, including all projected fields.
- `orgizepy.functions.headline_functions` reads the Scheme-AOT headline functions
  from a parsed document.
- `orgizepy.contract.evaluate_contract` calls the independent Scheme/Gambit
  Contract ABI over explicitly admitted Element rows.

The parser and functions do not initialize Gerbil. The Contract runtime starts
once per process on the main thread and shuts down on process exit; Gambit cannot be restarted in
the same process after shutdown. Contract evaluation does not
silently flatten a parsed Element's multiple fields into the narrower Contract
ABI row shape. The standalone Contract library is bundled in platform wheels;
source-tree tests can set `ORGIZE_CONTRACT_LIBRARY` to its built path.

The project is managed by `uv`; Rust bindings are built with PyO3 and maturin.
