set dotenv-load := false

default:
    @just --list

wasm-build:
    git submodule update --init --recursive wasm
    cd wasm && CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=../target/orgize-wasm wasm-pack build -t web -d dist --out-name orgize
    rm -f wasm/dist/.gitignore

wasm: wasm-build

wasm-clean:
    rm -rf wasm/dist
