# CI prepares the same compiler-owned program used by Rust and maturin.
# Package installation owns transitive POO/core dependencies; the parser SDK
# itself is rebuilt from Cargo's declared immutable checkout, not an old lib.
[group('checks')]
native-ci-prepare: native-ci-tools-check
    gxpkg deps --install
    just native-ci-program-owner "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-parser-runtime") | .manifest_path | sub("/rust/gerbil-parser-runtime/Cargo.toml$"; "")')" "{{ justfile_directory() }}/target/native-ci"

# Fail before building transitive packages when a runner lacks owner tools.
[private]
native-ci-tools-check:
    command -v rg
    command -v jq
    command -v python3
    command -v cargo
    command -v git
    command -v gerbil
    command -v gxi
    command -v gxpkg

[private]
native-ci-program-owner parser_source output:
    just native-parser-revision-check "{{ parser_source }}" "{{ parser_revision }}"
    just scheme-aligned-parser-build-receipt "{{ parser_source }}" "{{ output }}/parser" "{{ env_var('GERBIL_PATH') }}/lib"
    just scheme-parser-build-isolated "{{ output }}/org" "{{ output }}/parser/lib:{{ env_var('GERBIL_PATH') }}/lib"
    just scheme-aligned-program-stage-receipt "{{ justfile_directory() }}/target/gerbil-parser" "{{ output }}/org/lib:{{ output }}/parser/lib:{{ env_var('GERBIL_PATH') }}/lib" "{{ output }}/parser" "{{ output }}/org"

[group('checks')]
native-ci-scheme-test:
    just scheme-native-closure-receipt "{{ justfile_directory() }}/target/native-ci/closure" "{{ justfile_directory() }}/target/native-ci/org/lib:{{ justfile_directory() }}/target/native-ci/parser/lib:{{ env_var('GERBIL_PATH') }}/lib"

# Reuse the same Darwin/native linker policy as local qualification.
[group('checks')]
native-ci-rust-test features:
    {{ native_env }} {{ native_rust_flags }} cargo test --locked --workspace --all-targets --no-default-features --features "{{ features }}"

[private]
native-ci-scenario-test runtime target filter:
    {{ native_env }} {{ native_rust_flags }} cargo test --locked --release --no-default-features --features "{{ runtime }}" {{ if target == "lib" { "--lib" } else { "--test " + target } }} "{{ filter }}" -- --ignored --nocapture

[group('checks')]
native-ci-clippy features:
    {{ native_env }} {{ native_rust_flags }} cargo clippy --locked --workspace --all-targets --no-default-features --features "{{ features }}" -- -D warnings
