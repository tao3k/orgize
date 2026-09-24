set dotenv-load := false

default:
    @just --list

# Development-only: both arguments are compiled Gerbil lib directories.
# Cargo consumers do not need Gerbil or this recipe.
scheme-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test languages/org/v1/parser-test.ss languages/org/v1/graph-test.ss languages/org/v1/modules/org-elements/elements-module-test.ss tests/fixtures/org-elements/customer-query-test.ss languages/org/v1/modules/org-contract/parser-test.ss languages/org/v1/modules/org-contract/contract-test.ss

wasm-build:
    git submodule update --init --recursive wasm
    cd wasm && CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=../target/orgize-wasm wasm-pack build -t web -d dist --out-name orgize
    rm -f wasm/dist/.gitignore

wasm: wasm-build

wasm-clean:
    rm -rf wasm/dist
