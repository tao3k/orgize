# Compilation uses the qualified 2 GiB heap; runtime tests keep scheme_gambopt.
ci_build_gambopt := replace(scheme_gambopt, "max-heap=1G", "max-heap=2G")

# CI prepares the same compiler-owned program used by Rust and maturin.
# Package installation owns Parser and inherited POO/Core dependencies.
# Cargo exclusively selects the bridge source; compile that exact checkout into
# the existing Gerbil output before compiling the Orgize program.
[group('checks')]
ci-prepare: ci-dependencies ci-verify-dependencies ci-build-program

[group('checks')]
ci-build-provider:
    {{ platform_env }} {{ platform_rust_flags }} cargo build --locked --release --bin orgize

[group('checks')]
ci-dependencies:
    command -v gerbil
    just public-parser-interface-check
    # Let the package manager finish installing before starting the editor index.
    {{ scheme_env }} GAMBOPT="{{ package_gambopt }}" gerbil interactive -e '(import (only-in :gerbil/tools/gxpkg pkg-install pkg-plist with-pkg-build-env)) (with-pkg-build-env (for-each pkg-install (pgetq depend: (pkg-plist ".") (quote ()))))'
    # Keep the full index in one process because the Linux release's index worker can crash.
    cd "{{ env_var('GERBIL_PATH') }}/pkg" && { {{ scheme_env }} GERBIL_BUILD_CORES=1 GAMBOPT="{{ package_gambopt }}" gxtags . > ../dependency-index.log 2>&1 || { status=$?; tail -n 80 ../dependency-index.log; exit "$status"; }; }

[group('checks')]
ci-verify-dependencies:
    #!/usr/bin/env bash
    set -euo pipefail
    just public-parser-interface-check
    package_root="{{ env_var('GERBIL_PATH') }}/pkg/github.com/tao3k"
    for package in gerbil-parser gebril-yuanxu; do
        dependency=$(rg -o "github\.com/tao3k/$package@[0-9a-f]{40}" gerbil.pkg)
        checkout="$package_root/$package"
        revision="${dependency##*@}"
        actual=$(git -C "$checkout" rev-parse HEAD)
        expected=$(git -C "$checkout" rev-parse --verify "$revision^{commit}")
        if [[ "$actual" != "$expected" ]]; then
            printf '%s resolves to %s, expected %s\n' "$dependency" "$actual" "$expected" >&2
            exit 1
        fi
        git -C "$checkout" diff --quiet HEAD --
    done

[group('checks')]
ci-build-program:
    just alignment-dependency-check "{{ env_var('GERBIL_PATH') }}/pkg/github.com/tao3k/gerbil-parser" "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')"
    cd "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')" && {{ scheme_env }} GAMBOPT="{{ ci_build_gambopt }}" {{ timeout_command }} --signal=TERM --kill-after=5s 600s gerbil build --optimized
    {{ scheme_env }} GAMBOPT="{{ ci_build_gambopt }}" {{ timeout_command }} --signal=TERM --kill-after=5s {{ if host_os == "macos" { "900s" } else { "600s" } }} gerbil build --optimized
    just parser-program-source-check "{{ justfile_directory() }}/target/gerbil-parser/program.json" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}"

# Restored products are admitted by the existing source and cohort owners.
[group('checks')]
ci-verify-prepared:
    just ci-verify-dependencies
    just alignment-dependency-check "{{ env_var('GERBIL_PATH') }}/pkg/github.com/tao3k/gerbil-parser" "$(cargo metadata --locked --format-version 1 | jq -r '.packages[] | select(.name == "gerbil-scheme") | .manifest_path | sub("/crates/gerbil-scheme/Cargo.toml$"; "")')"
    just parser-program-source-check "{{ justfile_directory() }}/target/gerbil-parser/program.json" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}" "{{ env_var('GERBIL_PATH') }}"

# gxtest owns discovery, execution and exit status. The observer only keeps
# the existing five-second output deadline; it cannot select or run Suites.
[group('checks')]
ci-scheme-test:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target/scheme-test
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" {{ timeout_command }} --foreground --signal=TERM --kill-after=3s 120s python3 tools/ci/watch-real-output.py {{ if host_os == "macos" { "env DYLD_PRINT_APIS=1" } else { "env" } }} gxi :gerbil-scheme-rust/scheme/build test "{{ justfile_directory() }}" orgize-test.ss 2>&1 | tee target/scheme-test/scheme-test.log
    test "$(rg -c '^CASE-OK ' target/scheme-test/scheme-test.log)" -eq 238
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

[group('checks')]
ci-rust-native-startup features:
    {{ platform_env }} {{ platform_rust_flags }} cargo test --locked --workspace --no-default-features --features "{{ features }}" --lib tests::explicit_startup_precedes_parallel_native_unit_cases -- --exact --nocapture

# Reuse the same Darwin/native linker policy as local qualification.
[group('checks')]
ci-rust-test features: (ci-rust-native-startup features)
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p target/contract-control
    contract_test_status=0
    {{ platform_env }} {{ platform_rust_flags }} cargo test --locked --workspace --tests --examples --no-default-features --features "{{ features }}" -- --skip tests::explicit_startup_precedes_parallel_native_unit_cases --nocapture 2>&1 | tee target/contract-control/tests.log || contract_test_status=$?
    if (( contract_test_status != 0 )); then
        if rg --quiet 'org query assertion fixture exceeded' target/contract-control/tests.log; then
            # Same debug configuration and case; diagnostic output cannot admit a failed gate.
            contract_diagnostic_status=0
            {{ platform_env }} {{ platform_rust_flags }} ORGIZE_NATIVE_CONSUMER_CASE=lint_contract::lint_contract_org_query_assertion_fixture_stays_in_millisecond_budget cargo test --locked --workspace --no-default-features --features "{{ features }},orgize/runtime-profile" --test integration_test consumer_fixture::explicit_startup_precedes_parallel_consumer_cases -- --exact --nocapture 2>&1 | tee target/contract-control/diagnostic.log || contract_diagnostic_status=$?
            printf 'CONTRACT-DIAGNOSTIC original_exit=%s diagnostic_exit=%s (diagnostic only)\n' "$contract_test_status" "$contract_diagnostic_status"
        fi
        exit "$contract_test_status"
    fi
    {{ platform_env }} {{ platform_rust_flags }} cargo bench --locked --workspace --no-default-features --features "{{ features }}" -- --test

[private]
ci-scenario-test runtime target filter:
    {{ platform_env }} {{ platform_rust_flags }} cargo test --locked --release --no-default-features --features "{{ runtime }}" {{ if target == "lib" { "--lib" } else { "--test " + target } }} "{{ filter }}" -- --ignored --nocapture

[group('checks')]
ci-clippy features:
    {{ platform_env }} {{ platform_rust_flags }} cargo clippy --locked --workspace --all-targets --no-default-features --features "{{ features }}" -- -D warnings
