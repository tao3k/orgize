# Compilation uses the qualified 2 GiB heap; runtime tests keep scheme_gambopt.
ci_build_gambopt := replace(scheme_gambopt, "max-heap=1G", "max-heap=2G")

# CI prepares the same compiler-owned program used by Rust and maturin.
# Package installation owns Parser and inherited POO/Core dependencies.
# Cargo exclusively selects the bridge source; compile that exact checkout into
# the existing Gerbil output before compiling the Orgize program.
[group('checks')]
ci-prepare: ci-dependencies ci-build-program

[group('checks')]
ci-dependencies:
    command -v gerbil
    just public-parser-interface-check
    {{ scheme_env }} GAMBOPT="{{ package_gambopt }}" gerbil deps --install

[group('checks')]
ci-build-program:
    just alignment-dependency-check "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-parser-runtime") | .manifest_path | sub("/rust/gerbil-parser-runtime/Cargo.toml$"; "")')" "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')"
    cd "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')" && {{ scheme_env }} GAMBOPT="{{ ci_build_gambopt }}" {{ timeout_command }} --signal=TERM --kill-after=5s 600s gerbil build --optimized
    {{ scheme_env }} GAMBOPT="{{ ci_build_gambopt }}" {{ timeout_command }} --signal=TERM --kill-after=5s {{ if host_os == "macos" { "900s" } else { "600s" } }} gerbil build --optimized
    just parser-program-source-check "{{ justfile_directory() }}/target/gerbil-parser/program.json" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}"

# Restored products are admitted by the existing source and cohort owners.
[group('checks')]
ci-verify-prepared:
    just public-parser-interface-check
    just alignment-dependency-check "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-parser-runtime") | .manifest_path | sub("/rust/gerbil-parser-runtime/Cargo.toml$"; "")')" "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')"
    just parser-program-source-check "{{ justfile_directory() }}/target/gerbil-parser/program.json" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}"

# gxtest owns discovery, execution and exit status. The observer only keeps
# the existing five-second output deadline; it cannot select or run Suites.
[group('checks')]
ci-scheme-test:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target/scheme-test
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" {{ timeout_command }} --foreground --signal=TERM --kill-after=3s 120s python3 tools/ci/watch-real-output.py {{ if host_os == "macos" { "env DYLD_PRINT_APIS=1" } else { "env" } }} gxi :gerbil-scheme-rust/scheme/build test "{{ justfile_directory() }}" orgize-test.ss 2>&1 | tee target/scheme-test/scheme-test.log
    test "$(rg -c '^CASE-OK ' target/scheme-test/scheme-test.log)" -eq 237
    test "$(rg -c '^SUITE-OK ' target/scheme-test/scheme-test.log)" -eq 19
    rg --quiet '^OK$' target/scheme-test/scheme-test.log
    if rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' target/scheme-test/scheme-test.log; then
        echo 'FAIL: Scheme test output contains an error marker' >&2
        exit 1
    fi
    status=0
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" {{ timeout_command }} --foreground --signal=TERM --kill-after=3s 120s python3 tools/ci/watch-real-output.py {{ if host_os == "macos" { "env DYLD_PRINT_APIS=1" } else { "env" } }} gxi :gerbil-scheme-rust/scheme/build test "{{ justfile_directory() }}" t/fixtures/cli-failure.ss > target/scheme-test/failure-control.log 2>&1 || status=$?
    test "$status" -eq 42
    rg --quiet '^ERROR CHECK ' target/scheme-test/failure-control.log
    ! rg --quiet '^OK$' target/scheme-test/failure-control.log

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
