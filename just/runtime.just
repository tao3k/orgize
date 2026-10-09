# runtime: native architecture task owner.

# Both tanglers are independent host executables with one startup owner each.
[private]
native-runtime-example-build manifest gsc runtime:
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --locked --no-default-features --features "{{ runtime }}" --profile release --no-run --example org_contract_tangle --example org_elements_tangle -vv

[group('runtime')]
native-runtime-example-build-receipt manifest gsc runtime output:
    mkdir -p "{{ output }}"
    just native-runtime-example-build "{{ manifest }}" "{{ gsc }}" "{{ runtime }}" 2>&1 | tee "{{ output }}/example-build.log" | rg '^   Compiling|^    Finished|^  Executable|^error|^warning: failed|^\[orgize .*orgize-native:.*state=(start|complete)'
    rg --quiet 'Finished .*release.* profile' "{{ output }}/example-build.log"
    test "$(rg -c '^  Executable' '{{ output }}/example-build.log')" -eq 2
    ! rg --quiet '^error:' "{{ output }}/example-build.log"

[group('runtime')]
native-runtime-example-test-receipt binary suite count output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py "{{ binary }}" --nocapture 2>&1 | tee "{{ output }}/{{ suite }}.log"
    rg --quiet '^startup-native suite={{ suite }} concurrent-cases={{ count }} complete OK$' "{{ output }}/{{ suite }}.log"
    test "$(rg -c '^native-example case=.* OK$' '{{ output }}/{{ suite }}.log')" -eq "{{ count }}"
    rg --quiet '^test result: ok\. 1 passed; 0 failed;' "{{ output }}/{{ suite }}.log"
    ! rg --quiet 'FAILED|panicked at|^error:' "{{ output }}/{{ suite }}.log"

[group('runtime')]
native-runtime-example-failure-control binary case output:
    mkdir -p "{{ output }}"
    status=0; ORGIZE_NATIVE_EXAMPLE_CASE_PANIC="{{ case }}" python3 tools/ci/watch-real-output.py "{{ binary }}" --nocapture 2>&1 | tee "{{ output }}/example-failure-control.log" || status=$?; test "$status" -eq 101
    rg --quiet '^native-example case={{ case }} FAILED$' "{{ output }}/example-failure-control.log"
    rg --quiet '^test result: FAILED\. 0 passed; 1 failed;' "{{ output }}/example-failure-control.log"
    ! rg --quiet '^startup-native .* complete OK$' "{{ output }}/example-failure-control.log"

# Private queue synchronization only; native parser qualification stays separate.
[private]
native-runtime-queue-test-receipt gsc output sanitizer="":
    mkdir -p "{{ output }}"
    {{ native_env }} cc -std=c11 -O2 -g -pthread {{ sanitizer }} -I"{{ parent_directory(parent_directory(gsc)) }}/include" -Ibindings/c/include bindings/c/tests/orgize-runtime-queue-test.c -o "{{ output }}/queue-test"
    python3 tools/ci/watch-real-output.py "{{ output }}/queue-test" 2>&1 | tee "{{ output }}/queue-test.log"
    rg --quiet '^QUEUE-OK complete$' "{{ output }}/queue-test.log"

# Qualify one immutable feature build; run after compilation, not alongside it.
[group('runtime')]
native-runtime-closure-test-receipt library consumers cutover index output runtime program:
    [[ "{{ runtime }}" == runtime-rust || "{{ runtime }}" == runtime-scheme ]]
    [[ "{{ program }}" =~ ^sha256:[0-9a-f]{64}$ ]]
    just native-runtime-library-test-receipt "{{ library }}" "{{ output }}/library" {{ if runtime == "runtime-rust" { "35" } else { "34" } }} 4
    just native-runtime-consumer-catalog-receipt "{{ consumers }}" "{{ output }}/consumers"
    rg --fixed-strings --quiet 'CONTRACT-CONTROL backend={{ runtime }} program={{ program }} ' "{{ output }}/consumers/consumer-catalog.log"
    just native-runtime-cutover-test-receipt "{{ cutover }}" "{{ output }}"
    just native-index-test-receipt "{{ index }}" "{{ output }}/index"
    just native-runtime-query-scale-receipt "{{ library }}" 1000 "{{ output }}/query-1000"
    just native-runtime-query-scale-receipt "{{ library }}" 10000 "{{ output }}/query-10000"

[private]
native-runtime-cutover-test-receipt binary output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py "{{ binary }}" --nocapture 2>&1 | tee "{{ output }}/cutover.log"
    rg --quiet '^startup-native concurrent-cases=11 complete OK$' "{{ output }}/cutover.log"
    rg --quiet '^test result: ok\. 1 passed; 0 failed;' "{{ output }}/cutover.log"
    ! rg --quiet '^FAIL:|panicked at|^test result: FAILED' "{{ output }}/cutover.log"

# Exact test counts are candidate inputs, not historical migration constants.
[group('runtime')]
native-runtime-library-test-receipt binary output ordinary_passed="26" ignored="2":
    mkdir -p "{{ output }}"
    "{{ binary }}" --list > "{{ output }}/library-catalog.log"
    test "$(rg -c ': test$' '{{ output }}/library-catalog.log')" -eq "$(({{ ordinary_passed }} + {{ ignored }} + 1))"
    just native-runtime-library-test-run "{{ binary }}" 2>&1 | tee "{{ output }}/library-test.log"
    rg --quiet '^startup-native unit-cases=85 complete OK$' "{{ output }}/library-test.log"
    rg --quiet '^test result: ok\. {{ ordinary_passed }} passed; 0 failed; {{ ignored }} ignored;' "{{ output }}/library-test.log"
    ! rg --quiet '^test .* \.\.\. FAILED$|^test result: FAILED|^failures:$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=query_pushdown_preserves_complete_native_projection OK$' "{{ output }}/library-test.log"
    rg --quiet '^test document::query_match::tests::ascii_search_matches_allocating_reference_at_utf8_boundaries \.\.\. ok$' "{{ output }}/library-test.log"
    rg --quiet '^test document::query_match::tests::prepared_predicates_preserve_field_and_term_boundaries \.\.\. ok$' "{{ output }}/library-test.log"

[group('runtime')]
native-index-test-receipt binary output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py "{{ binary }}" explicit_startup_precedes_parallel_org_native_index_handoff_cases --exact --nocapture 2>&1 | tee "{{ output }}/native-index-test.log"
    rg --quiet '^startup-native suite=org_native_index_handoff concurrent-cases=23 complete OK$' "{{ output }}/native-index-test.log"
    rg --quiet '^test result: ok\. 1 passed; 0 failed;' "{{ output }}/native-index-test.log"
    ! rg --quiet '^FAIL:|^error:|panicked at|^test result: FAILED' "{{ output }}/native-index-test.log"

# Separate same-program ASP legs: a failed gate never hides the other leg.
[private]
native-runtime-query-batch-run binary:
    "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate --list | rg 'document_query_org_elements_aot_stays_inside_scenario_gate: test$'
    "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate --ignored --nocapture --test-threads=1

[private]
native-runtime-query-individual-control-run binary:
    "{{ binary }}" document_query_org_elements_aot_individual_control --list | rg 'document_query_org_elements_aot_individual_control: test$'
    "{{ binary }}" document_query_org_elements_aot_individual_control --ignored --nocapture --test-threads=1

# Select a qualified compiler-owned closure without rewriting its module paths.
# Keep the previous default manifest recoverable; its old modules are untouched.
[private]
native-parser-program-select manifest previous_label:
    jq --exit-status '.schema == "gerbil-scheme-rust.aot-program.v1" and (.modules | length > 0) and (.stub | type == "string")' "{{ manifest }}"
    mkdir -p target/native-program-history target/gerbil-parser
    if test -f target/gerbil-parser/program.json; then cp -n target/gerbil-parser/program.json "target/native-program-history/{{ previous_label }}.json"; fi
    cp "{{ manifest }}" target/gerbil-parser/program.json
    cmp "{{ manifest }}" target/gerbil-parser/program.json

[private]
native-build-test bridge_source filter="native_command":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" cargo test --offline --manifest-path "{{ bridge_source }}/Cargo.toml" -p gerbil-scheme-native-build --lib "{{ filter }}" -- --nocapture

[private]
native-owner-program-build source:
    mkdir -p "{{ source }}/.gerbil"
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="$PWD/.gerbil" gerbil compile -V -O scheme/native.ss scheme/program-build.ss 2>&1 | tee "$PWD/.gerbil/alignment-build.log" | rg '^compile |^invoke |^\*\*\* ERROR|^--- Syntax Error'

# Consume the pinned upstream native-build owner.
[private]
native-parser-test-owner manifest gsc test="gerbil_native_cutover" runtime="runtime-rust":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --test "{{ test }}" -- --nocapture

[private]
native-parser-check-owner manifest gsc:
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo check --offline -vv --tests --benches

[private]
native-parser-unit-owner manifest gsc:
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --lib org_aot::native_events::tests -- --nocapture

[private]
native-runtime-cold-owner manifest gsc runtime="runtime-rust" profile="release" samples="10":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo run --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --example runtime_cold -- "{{ samples }}"

# Explicit local Apple ld recursion diagnostic. Only the selected final target
# changes linker implementation; profiles, Thin LTO and dead-strip stay intact.
# Cargo also builds CLI targets for benches/tests, so the explicit diagnostic
# covers the whole Apple target invocation, not only the final selected target.
# This is not a production/runtime fallback, nor a portable package receipt.
[private]
native-runtime-link-owner manifest gsc runtime kind name profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo rustc --offline -vv --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" "{{ kind }}" "{{ name }}"

# Build the explicit-startup integration suites; execution remains separately bounded.
[private]
native-runtime-integration-build-owner manifest gsc runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --no-run --test gerbil_native_cutover --test native_host_children --test org_cutover_parity --test org_native_index_handoff --test org_aot_owned_projection

# Preserve an already-qualified executable before Cargo replaces its artifact.
[private]
native-runtime-binary-save binary label:
    mkdir -p target/runtime-receipt-binaries
    cp -n "{{ binary }}" "target/runtime-receipt-binaries/{{ label }}"

[private]
native-query-algorithm-runtime-comparison-receipt output:
    just native-runtime-parallel-compare "{{ output }}/rust-matrix/matrix.json" "{{ output }}/scheme-matrix/matrix.json" 2>&1 | tee "{{ output }}/runtime-comparison.log"

[private]
native-runtime-parallel-build-owner manifest gsc runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo bench --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --bench org_runtime --no-run

[private]
native-runtime-test-run binary filter="":
    "{{ binary }}" "{{ filter }}" --nocapture

[private]
native-runtime-consumer-test-run binary case="":
    ORGIZE_NATIVE_CONSUMER_CASE="{{ case }}" just native-runtime-test-run "{{ binary }}"

# Compile the libFuzzer-native startup hook; execution is a separate gate.
[group('runtime')]
native-runtime-fuzz-check manifest gsc:
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo check --offline --locked --manifest-path fuzz/Cargo.toml

# Optional-feature admission uses the same host startup and linker policy.
[private]
native-runtime-consumer-feature-build-owner manifest gsc runtime features profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --locked --profile "{{ profile }}" --no-default-features --features "{{ runtime }},{{ features }}" --no-run --test integration_test -vv

[group('runtime')]
native-runtime-consumer-feature-build-receipt manifest gsc native_output output runtime features profile="release":
    mkdir -p "{{ output }}"
    ORGIZE_GERBIL_NATIVE_OUTPUT="{{ native_output }}" just native-runtime-consumer-feature-build-owner "{{ manifest }}" "{{ gsc }}" "{{ runtime }}" "{{ features }}" "{{ profile }}" 2>&1 | tee "{{ output }}/feature-build.log"
    rg --quiet 'Finished .*profile' "{{ output }}/feature-build.log"
    test "$(rg -c '^  Executable ' "{{ output }}/feature-build.log")" -eq 1
    ! rg --quiet '^error:|^error\[' "{{ output }}/feature-build.log"

# Focused diagnostic only; never replaces the full concurrent catalog gate.
# Record host contention without changing or stopping unrelated workloads.
[group('runtime')]
native-runtime-host-load-receipt output:
    mkdir -p "{{ output }}"
    uptime | tee "{{ output }}/host-load.log"
    ps -Ao pid=,pcpu=,etime=,comm= | LC_ALL=C sort -k2,2nr | awk 'NR <= 20' | tee -a "{{ output }}/host-load.log"

[private]
native-runtime-consumer-control-receipt binary case output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py just native-runtime-consumer-test-run "{{ binary }}" "{{ case }}" 2>&1 | tee "{{ output }}/consumer-control.log"
    rg --quiet '^startup-native consumer-cases=1 callers=[1-9][0-9]* complete OK$' "{{ output }}/consumer-control.log"
    rg --quiet '^test result: ok\.' "{{ output }}/consumer-control.log"
    ! rg --quiet 'FAILED|panicked at|^FAIL:' "{{ output }}/consumer-control.log"

# Keep every failed sample; alternating controls are diagnostic, not a p99 claim.
[private]
native-runtime-consumer-matched-control reference candidate case output samples="5":
    test "{{ samples }}" -ge 1
    failed=0; for ((sample=1; sample<={{ samples }}; sample++)); do if ! just native-runtime-consumer-control-receipt "{{ reference }}" "{{ case }}" "{{ output }}/reference-$sample"; then failed=1; fi; if ! just native-runtime-consumer-control-receipt "{{ candidate }}" "{{ case }}" "{{ output }}/candidate-$sample"; then failed=1; fi; done; test "$failed" -eq 0

# Empty selection retains the fixture's exact original-catalog assertion.
[group('runtime')]
native-runtime-consumer-catalog-receipt binary output:
    mkdir -p "{{ output }}"
    just native-runtime-consumer-test-run "{{ binary }}" "" 2>&1 | tee "{{ output }}/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=[1-9][0-9]* callers=[1-9][0-9]* complete OK$' "{{ output }}/consumer-catalog.log"
    rg --quiet '^test result: ok\.' "{{ output }}/consumer-catalog.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/consumer-catalog.log"

# Same libtest binary in two processes: exclusive native startup must not race
# the pure/policy tests' stdio or host work. All 54 native cases stay concurrent.
[private]
native-runtime-library-test-run binary:
    "{{ binary }}" tests::explicit_startup_precedes_parallel_native_unit_cases --exact --nocapture
    "{{ binary }}" --skip tests::explicit_startup_precedes_parallel_native_unit_cases --nocapture

[private]
native-runtime-query-scenario-receipt binary output:
    mkdir -p "{{ output }}"
    "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate --list | rg '^document::org_elements_query_project_tests::document_query_org_elements_aot_stays_inside_scenario_gate: test$'
    just native-runtime-ignored-test-run "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate 2>&1 | tee "{{ output }}/query-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/query-scenario.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/query-scenario.log"

[private]
native-query-algorithm-contract-receipt binary output:
    mkdir -p "{{ output }}"
    "{{ binary }}" workspace_contract_scale_scenario_stays_in_budget --list | rg '^contract_workspace::workspace_contract_scale_scenario_stays_in_budget: test$'
    just native-runtime-ignored-test-run "{{ binary }}" workspace_contract_scale_scenario_stays_in_budget 2>&1 | tee "{{ output }}/contract-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/contract-scenario.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/contract-scenario.log"

# Diagnostic only: preserve the initial failed admission and every one of five
# alternating reference/candidate pairs. Never stop at the first green sample.
[private]
native-runtime-query-matched-control reference candidate output:
    failed=0; for sample in 1 2 3 4 5; do if ! just native-runtime-query-scenario-receipt "{{ reference }}" "{{ output }}/reference-$sample"; then failed=1; fi; if ! just native-runtime-query-scenario-receipt "{{ candidate }}" "{{ output }}/candidate-$sample"; then failed=1; fi; done; test "$failed" -eq 0

# Qualify the linked parser without depending on the checkout as the cwd.
[private]
native-runtime-library-detached-test-run binary directory:
    cd "{{ directory }}" && "{{ binary }}" tests::explicit_startup_precedes_parallel_native_unit_cases --exact --nocapture

# Existing ignored performance scenarios retain their original limits.
[private]
native-runtime-ignored-test-run binary filter:
    "{{ binary }}" "{{ filter }}" --ignored --nocapture

# Real project Query; count is explicit and the libtest inventory is checked.
# Qualification has no invented large-project latency pass threshold.
[private]
native-runtime-query-scale-run binary documents:
    "{{ binary }}" document_query_native_parallel_scale_qualification --list | rg 'document_query_native_parallel_scale_qualification: test$'
    ORGIZE_QUERY_WORKER_TRACE=1 ORGIZE_QUERY_SCALE_DOCUMENTS="{{ documents }}" "{{ binary }}" document_query_native_parallel_scale_qualification --ignored --nocapture --test-threads=1

[group('runtime')]
native-runtime-query-scale-receipt binary documents output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py just native-runtime-query-scale-run "{{ binary }}" "{{ documents }}" 2>&1 | tee "{{ output }}/query-scale.log"
    rg --quiet '^QUERY-SCALE complete .*documents={{ documents }} .*parity=full-projection' "{{ output }}/query-scale.log"
    rg --quiet '^QUERY-SCALE filtered-complete query=no-hit documents={{ documents }} results=0 parity=full-projection' "{{ output }}/query-scale.log"
    rg --quiet '^QUERY-SCALE filtered-complete query=hit documents={{ documents }} results={{ documents }} parity=full-projection' "{{ output }}/query-scale.log"
    rg --quiet '^test result: ok\.' "{{ output }}/query-scale.log"
    ! rg --quiet '^FAIL:|FAILED|panicked at' "{{ output }}/query-scale.log"

# Supervisor never initializes native runtime; each case is a fresh child.
[private]
native-runtime-lifecycle-run binary:
    "{{ binary }}"

[private]
native-runtime-cold-run binary samples="10":
    "{{ binary }}" "{{ samples }}"

[group('runtime')]
native-parser-dependency-update:
    cargo fetch

# Resolve the declared immutable revision through the normal dependency graph.
[private]
native-parser-lock-update:
    cargo update -p gerbil-parser-runtime -p gerbil-parser-artifact
    cargo update --manifest-path fuzz/Cargo.toml -p gerbil-parser-runtime -p gerbil-parser-artifact

# Normal pinned upstream dependencies, without local bridge overrides.
[group('runtime')]
native-runtime-check-owner manifest gsc runtime="runtime-rust" profile="dev" verbosity="":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo check {{ verbosity }} --offline --locked --profile "{{ profile }}" --no-default-features --features "{{ runtime }}" --tests --benches --examples

# Source-only admission; this does not replace linked runtime receipts.
[private]
native-runtime-source-check-receipt manifest gsc native_output output runtime:
    mkdir -p "{{ output }}"
    ORGIZE_GERBIL_NATIVE_OUTPUT="{{ native_output }}" just native-runtime-check-owner "{{ manifest }}" "{{ gsc }}" "{{ runtime }},runtime-profile" release -vv 2>&1 | tee "{{ output }}/release-build.log"
    just native-semantic-wrapper-warning-check "{{ output }}"

[private]
native-runtime-consumer-build-owner manifest gsc runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --no-run --lib --tests

[private]
native-runtime-library-build-owner manifest gsc runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --no-run --lib -vv

[private]
native-runtime-library-build-receipt manifest gsc output runtime="runtime-rust" profile="release":
    mkdir -p "{{ output }}"
    just native-runtime-library-build-owner "{{ manifest }}" "{{ gsc }}" "{{ runtime }}" "{{ profile }}" 2>&1 | tee "{{ output }}/library-build.log"
    rg --quiet '^    Finished ' "{{ output }}/library-build.log"
    test "$(rg -c '^  Executable ' "{{ output }}/library-build.log")" -eq 1
    ! rg --quiet '^error:|^error\[' "{{ output }}/library-build.log"

# Build one matched candidate for native units, the full consumer catalog and
# scale receipts, public native cutover and index handoff, including the CLI.
# Qualification enables stage observations; performance builds stay separate.
[private]
native-runtime-closure-build-owner manifest gsc runtime="runtime-rust" profile="release" include_bench="true":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --locked --no-default-features --features "{{ runtime }},runtime-profile" --profile "{{ profile }}" --no-run --lib --test integration_test --test gerbil_native_cutover --test org_native_index_handoff {{ if include_bench == "true" { "--bench org_runtime" } else { "" } }} -vv

# Retain the exact Cargo build transcript; pipefail preserves the owner failure.
[group('runtime')]
native-runtime-closure-build-receipt manifest gsc native_output output runtime="runtime-rust":
    mkdir -p "{{ output }}"
    ORGIZE_GERBIL_NATIVE_OUTPUT="{{ native_output }}" just native-runtime-closure-build-owner "{{ manifest }}" "{{ gsc }}" "{{ runtime }}" release true 2>&1 | tee "{{ output }}/release-build.log"
    rg --quiet 'Finished .*release.*profile' "{{ output }}/release-build.log"
    test "$(rg -c '^  Executable ' "{{ output }}/release-build.log")" -eq 5
    ! rg --quiet '^error:|^error\[' "{{ output }}/release-build.log"
