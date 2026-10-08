# benchmark: native architecture task owner.

# Measure the existing public-entry Criterion scenarios, not a new harness.
[private]
native-parser-bench-owner manifest gsc profile="release" filter='Org::parse/(doc|plain-links|quote-heavy)\.org':
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo bench --offline --profile "{{ profile }}" --bench parse -- "{{ filter }}" --sample-size 10 --warm-up-time 0.2 --measurement-time 1 --noplot

# Reuse Criterion's committed input files and ASP's Scheme timing/memory owner.
# Matched execution-owner lanes; no parser/backend substitution.
[private]
native-runtime-bench-owner manifest gsc runtime="runtime-rust" profile="release":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo bench --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --bench org_runtime -- --sample-size 10 --warm-up-time 0.2 --measurement-time 1 --noplot

[private]
native-runtime-bench-run binary baseline="base":
    "{{ binary }}" --bench --sample-size 10 --warm-up-time 0.2 --measurement-time 1 --noplot --save-baseline "{{ baseline }}"

# Exactly N distinct documents per batch; does not use Criterion calibration.
[private]
native-runtime-corpus-run binary output documents="1000,10000" callers="1,8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_DOCS="{{ documents }}" ORGIZE_RUNTIME_CORPUS_CALLERS="{{ callers }}" ORGIZE_RUNTIME_CORPUS_REPEATS="{{ repeats }}" ORGIZE_RUNTIME_CORPUS_OUTPUT="{{ output }}" "{{ binary }}" --bench --noplot

[private]
native-runtime-corpus-compare rust_receipt scheme_receipt:
    jq --exit-status --slurp --from-file benches/support/runtime_corpus_compare.jq "{{ rust_receipt }}" "{{ scheme_receipt }}"

# Full scale is four distinct batches, not a small smoke mistaken for closure.
[private]
native-runtime-corpus-matrix-check receipt:
    jq --exit-status 'length == 4 and ((map([.documents, .callers, .repetition]) | sort) == [[1000,1,1],[1000,8,1],[10000,1,1],[10000,8,1]]) and all(.[]; .documents == .completed and .documents == .distinct_documents and .driver == "tokio") and ((map(.completed) | add) == 22000)' "{{ receipt }}"

[group('benchmark')]
native-runtime-tokio-corpus-run binary output documents="1000,10000" callers="1,8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_DRIVER=tokio just native-runtime-corpus-run "{{ binary }}" "{{ output }}" "{{ documents }}" "{{ callers }}" "{{ repeats }}"

# High-concurrency admissions plus independent native execution domains.
# The supervisor never initializes Gerbil; workers exec and warm up before timing.
[group('benchmark')]
native-runtime-parallel-corpus-run binary output documents="1000,10000" callers="64" domains="1,4,8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_DOMAINS="{{ domains }}" just native-runtime-corpus-run "{{ binary }}" "{{ output }}" "{{ documents }}" "{{ callers }}" "{{ repeats }}"

# Full native admission matrix with genuine completed-document progress.
[private]
native-query-algorithm-runtime-matrix-receipt binary output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py just native-runtime-parallel-corpus-run "{{ binary }}" "{{ output }}/matrix.json" 2>&1 | tee "{{ output }}/matrix.log"
    just native-runtime-parallel-matrix-check "{{ output }}/matrix.json"
    just native-runtime-profile-receipt-check "{{ output }}/matrix.json"
    ! rg --quiet '^FAIL:|panicked at' "{{ output }}/matrix.log"

[private]
native-runtime-parallel-matrix-check receipt:
    jq --exit-status 'length == 6 and ((map([.documents,.callers,.domains,.repetition]) | sort) == [[1000,64,1,1],[1000,64,4,1],[1000,64,8,1],[10000,64,1,1],[10000,64,4,1],[10000,64,8,1]]) and all(.[]; (.schema == "orgize.runtime-parallel.v1" or .schema == "orgize.runtime-parallel.v2" and .mode == "qualification") and .documents == .completed and .documents == .distinct_documents and .max_in_flight == .callers and .max_active_worker_requests == .domains and (.worker_pids | unique | length) == .domains and (.worker_completed | add) == .completed and all(.worker_completed[]; . > 0) and .worker_exit_success) and ((map(.completed) | add) == 33000)' "{{ receipt }}"

[group('benchmark')]
native-runtime-parallel-compare rust_receipt scheme_receipt:
    jq --exit-status --slurp --from-file benches/support/runtime_parallel_compare.jq "{{ rust_receipt }}" "{{ scheme_receipt }}"

[private]
native-runtime-parallel-report receipt:
    jq '[.[] | {backend, documents, callers, domains, completed, max_in_flight, max_active_worker_requests, documents_per_second, end_to_end_p95_ms: (.end_to_end_latency_ns.p95 / 1000000), end_to_end_p99_ms: (.end_to_end_latency_ns.p99 / 1000000)}]' "{{ receipt }}"

[private]
native-runtime-performance-run binary output documents="1000,10000" callers="64" domains="8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_MODE=performance just native-runtime-parallel-corpus-run "{{ binary }}" "{{ output }}" "{{ documents }}" "{{ callers }}" "{{ domains }}" "{{ repeats }}"

[private]
native-runtime-stage-report receipt:
    jq '[.[] | {backend,mode,documents,callers,domains,profile_enabled,documents_per_second,stage_mean_ms:(.stage_timings_ns | map_values(.mean / 1000000))}]' "{{ receipt }}"

[private]
native-runtime-owner-stage-report receipt:
    jq --exit-status --from-file benches/support/runtime_owner_stage_report.jq "{{ receipt }}"

[private]
native-runtime-transport-stage-scale-check receipt: (native-runtime-inner-stage-scale-check receipt)
    jq --exit-status 'all(.[]; .stage_timings_ns | has("native.owner_admission") and has("native.owner_service_inclusive") and has("native.result_copy") and has("native.completion_handoff"))' "{{ receipt }}"
    just native-runtime-transport-timing-check "{{ receipt }}"

[private]
native-runtime-transport-timing-check receipt:
    jq --exit-status 'length > 0 and all(.[]; .stage_timings_ns as $stages | all(["native.owner_admission","native.owner_service_inclusive","native.result_copy","native.completion_handoff"][]; $stages[.].total > 0 and $stages[.].mean > 0))' "{{ receipt }}"

[private]
native-runtime-transport-repeat-scale-check receipt: (native-runtime-profile-receipt-check receipt)
    jq --exit-status 'length == 4 and (map([.documents,.callers,.domains,.repetition]) | sort) == [[1000,64,8,1],[1000,64,8,2],[10000,64,8,1],[10000,64,8,2]] and (map(.completed) | add) == 22000 and all(.[]; .stage_timings_ns | has("native.scheme_fold") and has("native.tape_encode") and has("native.scheme_thread_cpu") and has("native.owner_admission") and has("native.owner_service_inclusive") and has("native.result_copy") and has("native.completion_handoff"))' "{{ receipt }}"
    just native-runtime-transport-timing-check "{{ receipt }}"

[private]
native-runtime-profile-receipt-check receipt:
    jq --exit-status 'length > 0 and all(.[]; .schema == "orgize.runtime-parallel.v2" and .profile_enabled and .completed == .documents and .distinct_documents == .documents and .max_in_flight == .callers and .max_active_worker_requests == .domains and .worker_exit_success and (.worker_pids | unique | length) == .domains and (.worker_completed | add) == .completed and (.mode == "performance" and .artifact == null or .mode == "qualification" and (.artifact | type) == "string") and (.stage_timings_ns | has("native.transport_inclusive") and has("artifact.syntax_index") and has("graph.project") and has("supervisor.queue")))' "{{ receipt }}"

[private]
native-runtime-profile-scale-check receipt: (native-runtime-profile-receipt-check receipt)
    jq --exit-status 'length == 2 and (map([.documents,.callers,.domains,.repetition]) | sort) == [[1000,64,8,1],[10000,64,8,1]] and (map(.completed) | add) == 11000' "{{ receipt }}"

# Semantic equality across an explicitly changed native program is distinct
# from the strict same-program artifact check and the runtime comparator.
[private]
native-runtime-qualification-semantic-check reference candidate:
    jq --exit-status --slurp '.[0] as $reference | (.[1] | length > 0) and all(.[1][]; . as $candidate | .mode == "qualification" and .completed == .documents and (.artifact | type) == "string" and any($reference[]; .documents == $candidate.documents and .corpus == $candidate.corpus and .artifact == $candidate.artifact))' "{{ reference }}" "{{ candidate }}"

[private]
native-runtime-qualification-artifact-check reference candidate:
    jq --exit-status --slurp '.[0] as $reference | all(.[1][]; . as $candidate | (.artifact | type) == "string" and any($reference[]; .documents == $candidate.documents and .corpus == $candidate.corpus and .program == $candidate.program and .artifact == $candidate.artifact))' "{{ reference }}" "{{ candidate }}"

# Retest an immutable executable; this gate is not a cross-program comparator.
[private]
native-runtime-control-identity-check reference candidate: (native-runtime-profile-receipt-check candidate)
    jq --exit-status --slurp '.[0] as $reference | (.[1] | length > 0) and all(.[1][]; . as $candidate | any($reference[]; .backend == $candidate.backend and .mode == $candidate.mode and .program == $candidate.program and .adapter == $candidate.adapter and .benchmark == $candidate.benchmark and .corpus == $candidate.corpus and .documents == $candidate.documents and .callers == $candidate.callers and .domains == $candidate.domains and .repetition == $candidate.repetition))' "{{ reference }}" "{{ candidate }}"

# Reject an unprofiled binary instead of accepting libtest's zero-test success.
[private]
native-runtime-query-profile-run binary:
    "{{ binary }}" document_query_org_elements_aot_profiles_existing_scenario_gate --list | rg 'document_query_org_elements_aot_profiles_existing_scenario_gate: test$'
    "{{ binary }}" document_query_org_elements_aot_profiles_existing_scenario_gate --ignored --nocapture --test-threads=1

[private]
scheme-event-benchmark parser_lib poo_flow_lib label="current" fixture="doc.org" output="":
    just scheme-native-executable-link t/benchmarks/org-event-fold.ss "{{ if output == "" { justfile_directory() + "/target/scheme-benchmark-receipts/" + label } else { output + ".native" } }}" "{{ parser_lib }}:{{ poo_flow_lib }}" org-event-fold
    just scheme-event-benchmark-run "{{ if output == "" { justfile_directory() + "/target/scheme-benchmark-receipts/" + label } else { output + ".native" } }}/org-event-fold" "{{ label }}" "{{ fixture }}" "{{ output }}"

[private]
scheme-event-benchmark-run binary label fixture output:
    test -f "{{ justfile_directory() }}/benches/fixtures/{{ fixture }}"
    mkdir -p "{{ if output == "" { justfile_directory() + "/target/scheme-benchmark-receipts/" + label } else { parent_directory(output) } }}"
    test ! -e "{{ if output == "" { justfile_directory() + "/target/scheme-benchmark-receipts/" + label + "/run.log" } else { output + ".run.log" } }}"
    env -u GERBIL_PATH -u GERBIL_LOADPATH GAMBOPT=max-heap=1G,debug=q python3 tools/ci/watch-real-output.py "{{ binary }}" "{{ label }}" "{{ justfile_directory() }}/benches/fixtures/{{ fixture }}" "{{ justfile_directory() }}/t/benchmarks/org-event-fold/benchmark.ss" "{{ output }}" 2>&1 | tee "{{ if output == "" { justfile_directory() + "/target/scheme-benchmark-receipts/" + label + "/run.log" } else { output + ".run.log" } }}"

[private]
scheme-name-set-benchmark parser_lib poo_flow_lib label output:
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-name-set.ss "{{ label }}" "{{ justfile_directory() }}/t/benchmarks/org-name-set/benchmark.ss" "{{ output }}"

[private]
scheme-name-set-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-name-set/compare.ss "{{ baseline }}" "{{ candidate }}"

[private]
scheme-helper-state-benchmark parser_lib poo_flow_lib label output mode="timing":
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-helper-state.ss "{{ label }}" "{{ justfile_directory() }}/t/benchmarks/org-helper-state/benchmark.ss" "{{ output }}" "{{ mode }}"

[private]
scheme-helper-state-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-helper-state/compare.ss "{{ baseline }}" "{{ candidate }}"

[private]
scheme-prefix-benchmark parser_lib poo_flow_lib label output:
    mkdir -p target/gerbil-test target/scheme-benchmark-receipts
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-prefix-probe.ss "{{ label }}" "{{ justfile_directory() }}/t/benchmarks/org-prefix-probe/benchmark.ss" "{{ output }}"

[private]
scheme-prefix-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-prefix-probe/compare.ss "{{ baseline }}" "{{ candidate }}"

[private]
scheme-event-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-event-fold/compare.ss "{{ baseline }}" "{{ candidate }}"

# Matches semantics and reports failed gates; does not qualify performance.
[private]
scheme-event-benchmark-diagnostic-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-event-fold/compare.ss "{{ baseline }}" "{{ candidate }}" diagnostic
