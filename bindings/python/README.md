# orgizepy

`orgizepy` is Orgize's Python SDK. It exposes the following APIs:

- `orgizepy.parser.parse_org` parses raw Org text using the Scheme-declared,
  native Gerbil AOT parser through the existing PyO3/Rust/native API. It returns
  typed Elements, including all projected fields.
- `orgizepy.functions.headline_functions` reads the Scheme-AOT headline functions
  from a parsed document.
- `orgizepy.edits` computes an exact-source digest and asks the Rust/Scheme-AOT
  path to validate source-bound edits. It returns candidate Org text only;
  authorization, review, source recheck, and persistence remain with the caller.
- `orgizepy.contract.evaluate_contract` calls the existing PyO3/Rust extension
  and Scheme Contract owner over explicitly admitted Element rows.

Parsing and Contract evaluation share the selected runtime embedded
in the maturin extension. `runtime-rust` (default) and `runtime-scheme` forward
the corresponding Orgize features; disable defaults to select Scheme.
Python has no CFFI loader, pointer buffers or separate runtime framework.
Only call-scoped or owned requests cross the selected owner boundary, so callers
may use worker threads. Contract evaluation still requires explicit admitted
rows; it does not silently flatten parsed Elements into the narrower ABI shape.
Parsing, Contract evaluation, source hashing and edit validation release the GIL for their pure
Rust/native work; Python objects are constructed/accessed with the GIL held.
Native request admission remains bounded by the selected owner, not by the GIL.
Builds require the native Gerbil program manifest and toolchain described in the
repository README. The standalone C library remains a separate C-consumer tool,
not a Python packaging prerequisite. No Python-level runtime shutdown is needed.

Import alone does not initialize the runtime. Explicitly initialize at exclusive
host startup, before starting workers or children and before parsing/contracts:

```python
from orgizepy import initialize_native_runtime
from orgizepy.parser import parse_org

initialize_native_runtime()
document = parse_org("* Native document\n")
```

During the first call, no live/new children, concurrent native initialization,
host stdio users or signal-disposition changes are allowed. Valid descriptors
0/1/2 are required. The linked parser must not use Scheme-owned stdio, terminals
or subprocesses. Host control signals and stdio flags are restored before any
requests are admitted; native heartbeat/processor signals remain runtime-owned.
Repeated calls return the original result, and failures are terminal. Python
does not enforce this process-wide admission contract: an already-running host
must not assume the GIL makes startup exclusive. This POSIX contract does not
qualify runtime restart, dynamic unloading or general host-signal isolation.

Shared C consumers must use ABI revision 2 and call
`orgize_shared_runtime_initialize()` under the same startup contract; C and
Python use the same owner and initializer, never a second runtime. Historical
lazy-startup failures are recorded in `docs/runtime-tokio-pr7c-20261003.org`.

Raw wheels retain build-host dynamic library paths. The `python-wheel-repair`
recipe relinks and bundles non-system dependencies; CI then tests the repaired
wheel after installation. On macOS the repair step also recalculates the
minimum supported OS tag from the bundled libraries.

The project is managed by `uv`; Rust bindings are built with PyO3 and maturin.
Bundled native and Rust dependencies are inventoried in
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md); the matching license texts
are included in each wheel and source distribution.
