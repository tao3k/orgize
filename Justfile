set dotenv-load := false
lib_ext := if os() == "macos" { "dylib" } else { "so" }
native_env := if os() == "macos" { "env -u SDKROOT CC=/usr/bin/cc CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/cc" } else { "env" }
rust_linker := if os() == "macos" { "-C linker=/usr/bin/cc" } else { "" }

default:
    @just --list

# Development-only: both arguments are compiled Gerbil lib directories.
# Cargo consumers do not need Gerbil or this recipe.
scheme-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test languages/org/v1/parser-test.ss languages/org/v1/line-event-parser-test.ss languages/org/v1/graph-test.ss languages/org/v1/modules/org-elements/elements-module-test.ss tests/fixtures/org-elements/customer-query-test.ss languages/org/v1/modules/org-contract/parser-test.ss languages/org/v1/modules/org-contract/contract-test.ss

# Standalone Scheme Contract ABI; ordinary Cargo parsing never needs it.
contract-library:
    {{ native_env }} python3 bindings/c/build-native-library.py --output target/liborgize.{{ lib_ext }}

contract-smoke: contract-library
    {{ native_env }} cc -I bindings/c/include bindings/c/tests/orgize-dynamic-harness.c -o target/orgize-dynamic-harness
    target/orgize-dynamic-harness target/liborgize.{{ lib_ext }}
    {{ native_env }} rustc --edition=2024 bindings/rust/native_smoke.rs -L native=target {{ rust_linker }} -o target/orgize-rust-smoke
    LD_LIBRARY_PATH=target target/orgize-rust-smoke

python-test: contract-library
    {{ native_env }} ORGIZE_CONTRACT_LIBRARY="{{ justfile_directory() }}/target/liborgize.{{ lib_ext }}" uv sync --directory bindings/python --locked --extra test
    {{ native_env }} ORGIZE_CONTRACT_LIBRARY="{{ justfile_directory() }}/target/liborgize.{{ lib_ext }}" uv run --directory bindings/python --offline pytest

python-wheel: contract-library
    {{ native_env }} python3 bindings/c/build-native-library.py --output bindings/python/src/orgizepy/lib/liborgize.{{ lib_ext }}
    {{ native_env }} uv build --directory bindings/python --wheel
    unzip -l bindings/python/dist/orgizepy-*.whl | grep 'orgizepy/lib/liborgize.{{ lib_ext }}'

wasm-build:
    git submodule update --init --recursive wasm
    cd wasm && CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=../target/orgize-wasm wasm-pack build -t web -d dist --out-name orgize
    rm -f wasm/dist/.gitignore

wasm: wasm-build

wasm-clean:
    rm -rf wasm/dist
