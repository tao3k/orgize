# development: native architecture task owner.

[private]
generate-contract-plan parser_lib poo_flow_lib source="languages/org/modules/org-contract/generated/contract-source.ss" output="languages/org/modules/org-contract/generated/contract-plan.rs":
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi languages/org/modules/org-contract/generate-plan.ss "{{ output }}" "{{ source }}"

# Standalone C consumers only; Python uses the shared maturin extension.
[private]
contract-library:
    {{ platform_env }} python3 bindings/c/build-library.py --output target/liborgize.{{ lib_ext }}

[group('development')]
contract-smoke: contract-library
    {{ platform_env }} cc -I bindings/c/include bindings/c/tests/orgize-dynamic-harness.c -o target/orgize-dynamic-harness
    target/orgize-dynamic-harness target/liborgize.{{ lib_ext }}
    {{ platform_env }} rustc --edition=2024 bindings/rust/smoke.rs -L native=target {{ rust_linker }} -o target/orgize-rust-smoke
    LD_LIBRARY_PATH=target target/orgize-rust-smoke

[private]
wasm-build:
    git submodule update --init --recursive wasm
    cd wasm && CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=../target/orgize-wasm wasm-pack build -t web -d dist --out-name orgize
    rm -f wasm/dist/.gitignore

[private]
wasm: wasm-build

[private]
wasm-clean:
    rm -rf wasm/dist
