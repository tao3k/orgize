# CI prepares the same compiler-owned program used by Rust and maturin.
# Package installation owns transitive POO/core dependencies; the parser SDK
# itself is rebuilt from Cargo's declared immutable checkout, not an old lib.
[group('checks')]
ci-prepare: ci-tools-check
    {{ scheme_env }} gxpkg deps --install
    just ci-program-owner "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-parser-runtime") | .manifest_path | sub("/rust/gerbil-parser-runtime/Cargo.toml$"; "")')" "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')" "{{ justfile_directory() }}/target/native-ci"

# Fail before building transitive packages when a runner lacks owner tools.
[private]
ci-tools-check:
    command -v rg
    command -v jq
    command -v python3
    command -v cargo
    command -v git
    command -v gerbil
    command -v gxi
    command -v gxpkg

[private]
ci-program-owner parser_source bridge_source output:
    just alignment-dependency-check "{{ parser_source }}" "{{ bridge_source }}"
    just scheme-bridge-build "{{ bridge_source }}" "{{ output }}/bridge"
    just scheme-aligned-parser-build-receipt "{{ parser_source }}" "{{ output }}/parser" "{{ env_var('GERBIL_PATH') }}/lib"
    just scheme-parser-build-isolated "{{ output }}/org" "{{ output }}/bridge/lib:{{ output }}/parser/lib:{{ env_var('GERBIL_PATH') }}/lib"
    just scheme-aligned-program-stage-receipt "{{ justfile_directory() }}/target/gerbil-parser" "{{ output }}/org/lib:{{ output }}/bridge/lib:{{ output }}/parser/lib:{{ env_var('GERBIL_PATH') }}/lib" "{{ output }}/parser" "{{ output }}/org" "{{ output }}/bridge"

[group('checks')]
ci-scheme-test:
    just scheme-closure-receipt "{{ justfile_directory() }}/target/native-ci/closure" "{{ justfile_directory() }}/target/native-ci/org/lib:{{ justfile_directory() }}/target/native-ci/parser/lib:{{ env_var('GERBIL_PATH') }}/lib"

# Reuse the same Darwin/native linker policy as local qualification.
[group('checks')]
ci-rust-test features:
    {{ platform_env }} {{ platform_rust_flags }} cargo test --locked --workspace --no-default-features --features "{{ features }}" --lib tests::explicit_startup_precedes_parallel_native_unit_cases -- --exact --nocapture
    {{ platform_env }} {{ platform_rust_flags }} cargo test --locked --workspace --tests --examples --no-default-features --features "{{ features }}" -- --skip tests::explicit_startup_precedes_parallel_native_unit_cases --nocapture
    {{ platform_env }} {{ platform_rust_flags }} cargo bench --locked --workspace --no-default-features --features "{{ features }}" -- --test

[private]
ci-scenario-test runtime target filter:
    {{ platform_env }} {{ platform_rust_flags }} cargo test --locked --release --no-default-features --features "{{ runtime }}" {{ if target == "lib" { "--lib" } else { "--test " + target } }} "{{ filter }}" -- --ignored --nocapture

[group('checks')]
ci-clippy features:
    {{ platform_env }} {{ platform_rust_flags }} cargo clippy --locked --workspace --all-targets --no-default-features --features "{{ features }}" -- -D warnings
