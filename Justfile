set dotenv-load := false
set shell := ["bash", "-euo", "pipefail", "-c"]
host_os := os()
lib_ext := if host_os == "macos" { "dylib" } else { "so" }
scheme_env := if host_os == "macos" { "env -u SDKROOT CC=/usr/bin/cc GERBIL_GCC=/usr/bin/cc" } else { "env" }
native_env := scheme_env + (if host_os == "macos" { " CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/cc" } else { "" })
native_rust_flags := if host_os == "macos" { "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS=\"-C linker=/usr/bin/cc -C link-arg=-Wl,-ld_classic\"" } else { "" }
rust_linker := if host_os == "macos" { "-C linker=/usr/bin/cc" } else { "" }
# Gerbil's loadable module resolves OS functions in its existing host process.
clock_linker := if host_os == "macos" { "-Wl,-undefined,dynamic_lookup" } else { "" }

# OS clock calibration is a native test, not a production parser dependency.
scheme-native-runtime-statistics-test output:
    test "{{ host_os }}" != "windows"
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-runtime-statistics.ss t/native-clock/reference.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib" gerbil test -v 3 t/native-runtime-statistics-test.ss 2>&1 | tee "{{ output }}/native-runtime-statistics-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/native-runtime-statistics-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/native-runtime-statistics-test.log"
    rg --quiet '^OK$' "{{ output }}/native-runtime-statistics-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/native-runtime-statistics-test.log"

scheme-native-clock-test output:
    test "{{ host_os }}" != "windows"
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss t/native-clock/reference.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib" gerbil test -v 3 t/native-clock-test.ss 2>&1 | tee "{{ output }}/native-clock-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/native-clock-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/native-clock-test.log"
    rg --quiet '^OK$' "{{ output }}/native-clock-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/native-clock-test.log"
repair_command := if host_os == "macos" { "delocate-wheel -w" } else { "auditwheel repair --wheel-dir" }

default:
    @just --list

# Bounded native migration records and implementation owner checklists.
native-document-layout-check:
    @awk 'FNR == 1 { if (NR > 1 && count > 500) { print "FAIL " previous " lines=" count; failed = 1 }; previous = FILENAME; count = 0 } { count++ } END { if (count > 500) { print "FAIL " previous " lines=" count; failed = 1 }; if (!failed) print "DOC-LAYOUT-OK: owner records <= 500 lines"; exit failed }' docs/20_parser/20.08_native_scheme_closure_ledger.org docs/20_parser/20.08_native_scheme_closure/*.org docs/90_operations/90.01_implementation_status.org docs/90_operations/90.01_implementation_status/*.org

# Validate platform-specific command expansion; this is not a Linux execution.
native-platform-command-check output:
    mkdir -p "{{ output }}"
    just --set host_os macos --dry-run scheme-citation-native-build /tmp/orgize-platform-check /tmp/orgize-loadpath 2>&1 | tee "{{ output }}/macos-scheme.log"
    just --set host_os linux --dry-run scheme-citation-native-build /tmp/orgize-platform-check /tmp/orgize-loadpath 2>&1 | tee "{{ output }}/linux-scheme.log"
    just --set host_os linux --dry-run scheme-native-clock-test /tmp/orgize-platform-check 2>&1 | tee "{{ output }}/linux-clock.log"
    just --set host_os linux --dry-run native-runtime-closure-build-owner /tmp/program.json /tmp/gsc /tmp/bridge runtime-rust release false 2>&1 | tee "{{ output }}/linux-rust.log"
    rg --quiet 'env -u SDKROOT CC=/usr/bin/cc' "{{ output }}/macos-scheme.log"
    ! rg --quiet 'SDKROOT|APPLE_DARWIN|/usr/bin/cc|-Wl,' "{{ output }}/linux-scheme.log" "{{ output }}/linux-clock.log" "{{ output }}/linux-rust.log"

# Development-only: both arguments are compiled Gerbil lib directories.
native-parser-revision-check source revision:
    test "$(git -C '{{ source }}' rev-parse HEAD)" = "{{ revision }}"
    rg --fixed-strings --quiet 'rev = "{{ revision }}"' Cargo.toml
    rg --fixed-strings --quiet 'git+https://github.com/tao3k/gerbil-parser?rev={{ revision }}#{{ revision }}' Cargo.lock
    rg --fixed-strings --quiet 'github.com/tao3k/gerbil-parser@{{ revision }}' gerbil.pkg
    git -C "{{ source }}" diff --check
    git -C "{{ source }}" status --short

# Integrated local admission; publication and global semantic closure are
# separate gates. Every consumer/control must belong to this exact program.
native-parser-alignment-admission-check source revision output program:
    just native-parser-revision-check "{{ source }}" "{{ revision }}"
    test "$(rg -c '^CASE-OK ' '{{ output }}/native-closure/native-test.log')" -eq 173
    test "$(rg -c '^SUITE-OK ' '{{ output }}/native-closure/native-test.log')" -eq 12
    rg --quiet '^OK$' "{{ output }}/native-closure/native-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/native-closure/native-test.log"
    just scheme-native-closure-failure-control "{{ output }}/native-closure"
    for runtime in rust scheme; do rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/$runtime-library/library-test.log"; rg --quiet '^test result: ok\. 24 passed; 0 failed;' "{{ output }}/$runtime-library/library-test.log"; rg --quiet '^startup-native consumer-cases=457 callers=12 complete OK$' "{{ output }}/$runtime-consumers/consumer-catalog.log"; rg --fixed-strings --quiet "CONTRACT-CONTROL backend=runtime-$runtime program={{ program }} " "{{ output }}/$runtime-consumers/consumer-catalog.log"; rg --fixed-strings --quiet "QUERY-CONTROL batched=true profile=false backend=runtime-$runtime program={{ program }} " "{{ output }}/$runtime-query/query-scenario.log"; rg --quiet '^test result: ok\.' "{{ output }}/$runtime-query/query-scenario.log"; rg --quiet '^startup-native suite=org_rowan_event_handoff concurrent-cases=23 complete OK$' "{{ output }}/$runtime-rowan/rowan-fixture-test.log"; rg --quiet '=+ 25 passed.*=+$' "{{ output }}/$runtime-python/installed-test.log"; ! rg --quiet 'FAILED|panicked at|^FAIL:' "{{ output }}/$runtime-consumers/consumer-catalog.log" "{{ output }}/$runtime-query/query-scenario.log" "{{ output }}/$runtime-rowan/rowan-fixture-test.log"; done
    for runtime in rust scheme; do jq --exit-status --arg program "{{ program }}" --arg backend "runtime-$runtime" 'all(.[]; .program == $program and .backend == $backend and .profile_enabled == false)' "{{ output }}/$runtime-matrix.json"; just native-runtime-parallel-matrix-check "{{ output }}/$runtime-matrix.json"; done
    just native-runtime-qualification-artifact-check "{{ output }}/rust-matrix.json" "{{ output }}/scheme-matrix.json"
    just native-runtime-parallel-compare "{{ output }}/rust-matrix.json" "{{ output }}/scheme-matrix.json"
    just native-document-layout-check

scheme-aligned-parser-build source output load_path:
    mkdir -p "{{ output }}"
    cd "{{ source }}" && {{ scheme_env }} GERBIL_BUILD_CORES=8 GERBIL_BUILD_VERBOSE=1 GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build.ss compile --optimized

# This semantic test lane does not perform native-link qualification.
scheme-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test t/*-test.ss
    just qualify-source-headlines "{{ justfile_directory() }}/target/gerbil-test" "{{ parser_lib }}:{{ poo_flow_lib }}"

# Source projection retains all four original cases under native gxtest.
qualify-source-headlines output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-elements/source-headlines.ss languages/org/v1/modules/org-elements/source-interface.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-source-headlines-qualification.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" python3 tools/ci/watch-real-output.py gerbil test -v 5 t/org-source-headlines-qualification.ss 2>&1 | tee "{{ output }}/module-test.log"
    test "$(rg -c '^CASE-OK ' '{{ output }}/module-test.log')" -eq 4
    rg --quiet '^OK$' "{{ output }}/module-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/module-test.log"

native-source-observation-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/publishing.rs tests/integration/semantic_ast/semantic_ast_projects_publishing.rs
    rustfmt --edition 2024 --config skip_children=true src/config.rs src/org_aot.rs src/org_element_query/mod.rs src/org_element_query/source_observation.rs src/semantic_ast/elements_bridge_model.rs tests/integration/org_element_query.rs tests/integration/lint_contract.rs benches/support/runtime_corpus.rs

# PR integration is not qualified by the earlier 454-case executables.
native-source-observation-catalog-check output:
    rg --quiet '^startup-native consumer-cases=457 callers=12 complete OK$' "{{ output }}/consumer-catalog.log"
    for name in named_query_observation_binds_graph_local_ids_to_exact_source named_query_observation_retains_effective_parse_configuration named_query_observation_rechecks_current_source_config_and_rule; do rg --quiet "^native-consumer case=org_element_query::$name OK$" "{{ output }}/consumer-catalog.log"; done
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/consumer-catalog.log"

# Publishing owner admission stays separate from whole-parser admission.
native-publishing-consumer-check output program:
    test -s "{{ output }}/normal/lib/static/orgize__languages__org__v1__modules__org-parser__publishing-value-funs.scm"
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite/publishing-tests/module-test.log')" -eq 6
    rg --quiet '^OK$' "{{ output }}/scheme-suite/publishing-tests/module-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/scheme-suite/publishing-tests/module-test.log"
    just native-publishing-runtime-check "{{ output }}" rust "{{ program }}"
    just native-publishing-runtime-check "{{ output }}" scheme "{{ program }}"

native-publishing-runtime-check output runtime program:
    just native-source-observation-catalog-check "{{ output }}/{{ runtime }}-consumers"
    rg --quiet '^native-consumer case=semantic_ast::semantic_ast_projects_publishing::semantic_ast_projects_publishing_settings_without_executing_export OK$' "{{ output }}/{{ runtime }}-consumers/consumer-catalog.log"
    rg --quiet --fixed-strings "CONTRACT-CONTROL backend=runtime-{{ runtime }} program={{ program }} " "{{ output }}/{{ runtime }}-consumers/consumer-catalog.log"
    rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/{{ runtime }}-units/library-test.log"
    rg --quiet --fixed-strings 'test result: ok. 24 passed; 0 failed;' "{{ output }}/{{ runtime }}-units/library-test.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/{{ runtime }}-units/library-test.log"

# This owner check does not replace whole-parser startup, performance or scale gates.
# Native gxtest remains the test owner; always compile the requested module.
scheme-native-module-fresh-test suite output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O "{{ suite }}"
    just scheme-native-module-test "{{ suite }}" "{{ output }}" "{{ load_path }}"

# Reuse an explicitly compiled module for isolated native startup diagnostics.
scheme-native-import-trace suite output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" python3 tools/ci/watch-real-output.py gerbil tools/ci/trace-native-import.ss -v 5 "{{ suite }}" 2>&1 | tee "{{ output }}/import-trace.log"

scheme-native-module-test suite output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" python3 tools/ci/watch-real-output.py gerbil test -v 5 "{{ suite }}" 2>&1 | tee "{{ output }}/module-test.log"
    rg --quiet '^CASE-OK ' "{{ output }}/module-test.log"
    rg --quiet '^OK$' "{{ output }}/module-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/module-test.log"

# Focus the existing Scheme-owned Org event suites with a pre-import heap cap.
scheme-event-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test -v 3 t/org-rowan-event-parser-test.ss 2>&1 | tee target/gerbil-test/org-event-parser.log
    rg --quiet '^MODULE-OK' target/gerbil-test/org-event-parser.log
    rg --quiet '^HARNESS-OK' target/gerbil-test/org-event-parser.log
    rg --quiet '^OK$' target/gerbil-test/org-event-parser.log
    ! rg --quiet 'ERROR|FAILED|FAILURE' target/gerbil-test/org-event-parser.log

# Compile the edited suite explicitly; import-only aggregate tests may resolve
# older compiled child suites from an existing dependency overlay.
native-list-opaque-format:
    rustfmt --edition 2024 --config skip_children=true tests/unit/aot_projection.rs

scheme-timestamp-cookie-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-inline-timestamp.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-inline.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-strategy.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/rowan-event-runtime.ss

scheme-source-switch-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-source-header.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-headline.ss languages/org/v1/modules/org-parser/event-include.ss languages/org/v1/modules/org-parser/event-inline.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-strategy.ss languages/org/v1/rowan-event-runtime.ss

native-source-switch-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/aot_block_switches.rs src/semantic_ast/aot_include_projection.rs src/semantic_ast/aot_projection/document.rs tests/unit/aot_projection.rs

scheme-semantic-content-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-source-content.ss languages/org/v1/modules/org-parser/event-table.ss languages/org/v1/modules/org-parser/event-table-formula.ss languages/org/v1/modules/org-parser/event-macro.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-headline.ss languages/org/v1/modules/org-parser/event-inline.ss languages/org/v1/modules/org-parser/event-strategy.ss languages/org/v1/rowan-event-runtime.ss

scheme-macro-runtime-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/rowan-event-runtime.ss languages/org/v1/modules/org-parser/macro-funs.ss

# Integrated semantic owners; the normal build graph lists the same roots.
scheme-semantic-owner-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/text-funs.ss languages/org/v1/modules/org-parser/block-line-funs.ss languages/org/v1/modules/org-parser/keyword-funs.ss languages/org/v1/modules/org-parser/dir-path-funs.ss

scheme-value-family-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-elements/headline-properties.ss languages/org/v1/modules/org-elements/link-properties.ss languages/org/v1/modules/org-elements/logbook-properties.ss languages/org/v1/modules/org-elements/table-properties.ss languages/org/v1/modules/org-elements/affiliated-properties.ss languages/org/v1/modules/org-elements/radio-match.ss languages/org/v1/modules/org-parser/family-funs.ss

scheme-lifecycle-value-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/text-funs.ss languages/org/v1/modules/org-parser/duration-funs.ss languages/org/v1/modules/org-parser/time-funs.ss languages/org/v1/modules/org-parser/unicode-context-data.ss languages/org/v1/modules/org-parser/unicode-lower-data.ss languages/org/v1/modules/org-parser/unicode-funs.ss languages/org/v1/modules/org-parser/property-token-funs.ss languages/org/v1/modules/org-parser/clock-window-funs.ss languages/org/v1/modules/org-parser/link-protocol-funs.ss languages/org/v1/modules/org-parser/source-value-funs.ss languages/org/v1/modules/org-parser/contract-reference-funs.ss languages/org/v1/modules/org-parser/source-header-policy-funs.ss languages/org/v1/modules/org-parser/metadata-value-funs.ss languages/org/v1/modules/org-parser/agenda-match-funs.ss languages/org/v1/modules/org-parser/interactive-value-funs.ss languages/org/v1/modules/org-parser/publishing-value-funs.ss languages/org/v1/modules/org-parser/citation-export-value-funs.ss languages/org/v1/modules/org-parser/value-funs.ss

native-dependency-owner-format:
    rustfmt --edition 2024 --config skip_children=true src/org_aot.rs src/semantic_ast/export_dependency_graph.rs src/semantic_ast/aot_projection/document.rs tests/unit/aot_projection/native_source_values.rs tests/integration/semantic_ast/semantic_ast_projects_m25_alignment.rs

native-citation-export-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/citation_export.rs src/semantic_ast/citation_export_native.rs tests/integration/semantic_ast/semantic_ast_projects_m25_alignment.rs

native-lifecycle-value-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/org_native_values.rs src/semantic_ast/property_model.rs src/semantic_ast/lifecycle_model.rs src/semantic_ast/agenda_time.rs src/semantic_ast/runtime_metadata.rs src/semantic_ast/settings.rs src/semantic_ast/aot_link_resolution.rs benches/support/runtime_corpus.rs
    rustfmt --edition 2024 --config skip_children=true tests/unit/aot_projection.rs tests/unit/aot_projection/native_lifecycle_values.rs tests/integration/org_headline_function_aot.rs
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/clock_table_time.rs src/semantic_ast/clock_table_properties.rs src/semantic_ast/property_profile.rs src/semantic_ast/progress.rs src/semantic_ast/link_protocols.rs src/semantic_ast/tag_vocabulary.rs src/semantic_ast/projection.rs

native-source-consumer-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/agenda_model.rs src/semantic_ast/sdd_model.rs src/semantic_ast/workspace_index.rs src/semantic_ast/org_interactive.rs
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/agenda_match.rs src/semantic_ast/org_native_values.rs src/semantic_ast/datetree.rs src/semantic_ast/special_properties.rs src/semantic_ast/column_views.rs src/semantic_ast/column_summaries.rs src/semantic_ast/clock_rollup.rs src/semantic_ast/property_schema.rs
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/includes.rs src/semantic_ast/source_block_headers.rs src/semantic_ast/source_block_references.rs src/semantic_ast/org_contract.rs src/semantic_ast/org_contract_model.rs src/semantic_ast/table_visualization.rs
    rustfmt --edition 2024 --config skip_children=true benches/support/runtime_corpus.rs tests/unit/aot_projection.rs tests/unit/aot_projection/native_source_values.rs
    rustfmt --edition 2024 --config skip_children=true tests/unit/org_contract_reference.rs tests/unit/lib.rs

native-source-consumer-library-test-receipt binary output:
    just native-value-family-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=native_source_values_preserve_presence_and_inert_syntax OK$' "{{ output }}/library-test.log"
    rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/library-test.log"

native-source-consumer-failure-preserve output:
    cp -n "{{ output }}/library-test.log" "{{ output }}/library-test-before-startup-migration.log"

native-source-consumer-query-profile-receipt binary output:
    mkdir -p "{{ output }}"
    just native-runtime-query-profile-run "{{ binary }}" 2>&1 | tee "{{ output }}/query-profile.log"
    rg --quiet '^QUERY-PROFILE sample=2 documents=48 stages_ns=' "{{ output }}/query-profile.log"
    rg --quiet '^test result: ok\.' "{{ output }}/query-profile.log"

# One cross-runtime gate; no individual helper smoke can select this program.
native-source-consumer-admission-check output rust_matrix scheme_matrix:
    rg --quiet '1f0d87f38079d1115148d4d3421cc95416813c94' Cargo.toml
    rg --quiet '1f0d87f38079d1115148d4d3421cc95416813c94' Cargo.lock
    rg --quiet '1f0d87f38079d1115148d4d3421cc95416813c94' gerbil.pkg
    test -s "{{ output }}/normal/lib/static/orgize__languages__org__v1__modules__org-parser__source-value-funs.scm"
    test -s "{{ output }}/normal/lib/static/orgize__languages__org__v1__modules__org-parser__contract-reference-funs.scm"
    test -s "{{ output }}/normal/lib/static/orgize__languages__org__v1__modules__org-parser__source-header-policy-funs.scm"
    test "$(rg -c '^CASE-OK ' '{{ output }}/source-tests/module-test.log')" -eq 5
    test "$(rg -c '^CASE-OK ' '{{ output }}/lifecycle-tests/module-test.log')" -eq 11
    test "$(rg -c '^CASE-OK ' '{{ output }}/event-tests/event-test.log')" -eq 103
    test "$(rg -c '^CASE-OK ' '{{ output }}/owner-tests/module-test.log')" -eq 9
    test "$(rg -c '^CASE-OK ' '{{ output }}/elements-tests/module-test.log')" -eq 14
    test "$(rg -c '^CASE-OK ' '{{ output }}/radio-tests/module-test.log')" -eq 5
    rg --quiet '^OK$' "{{ output }}/source-tests/module-test.log"
    rg --quiet '^OK$' "{{ output }}/lifecycle-tests/module-test.log"
    rg --quiet '^OK$' "{{ output }}/event-tests/event-test.log"
    rg --quiet '^OK$' "{{ output }}/owner-tests/module-test.log"
    rg --quiet '^OK$' "{{ output }}/elements-tests/module-test.log"
    rg --quiet '^OK$' "{{ output }}/radio-tests/module-test.log"
    rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/rust-library/library-test.log"
    rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/scheme-library/library-test.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/rust-consumers/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/scheme-consumers/consumer-catalog.log"
    rg --quiet '^native-consumer case=contract_evaluation::contract_org_property_scope_fixture_stays_in_millisecond_budget OK$' "{{ output }}/rust-contract/consumer-test.log"
    rg --quiet '^native-consumer case=contract_evaluation::contract_org_property_scope_fixture_stays_in_millisecond_budget OK$' "{{ output }}/scheme-contract/consumer-test.log"
    rg --quiet '^test result: ok\.' "{{ output }}/rust-query/query-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/scheme-query/query-scenario.log"
    just native-runtime-parallel-matrix-check "{{ rust_matrix }}"
    just native-runtime-parallel-matrix-check "{{ scheme_matrix }}"
    just native-runtime-qualification-artifact-check "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-runtime-parallel-compare "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-document-layout-check

# Integrated admission of metadata/Agenda through both existing runtime and
# Python front doors. Older candidates and chosen reruns cannot satisfy it.
native-metadata-consumer-admission-check output program:
    for config in Cargo.toml Cargo.lock gerbil.pkg; do rg --quiet '1f0d87f38079d1115148d4d3421cc95416813c94' "$config"; done
    test -s "{{ output }}/normal-v3/lib/static/orgize__languages__org__v1__modules__org-parser__metadata-value-funs.scm"
    test -s "{{ output }}/normal-v3/lib/static/orgize__languages__org__v1__modules__org-parser__agenda-match-funs.scm"
    test -s "{{ output }}/normal-v3/lib/static/orgize__languages__org__v1__modules__org-parser__interactive-value-funs.scm"
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/metadata-tests/module-test.log')" -eq 10
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/source-tests/module-test.log')" -eq 5
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/lifecycle-tests/module-test.log')" -eq 11
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/owner-tests/module-test.log')" -eq 9
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/elements-tests/module-test.log')" -eq 14
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/radio-tests/module-test.log')" -eq 5
    test "$(rg -c '^CASE-OK ' '{{ output }}/scheme-suite-v2/event-tests/event-test.log')" -eq 103
    for suite in metadata source lifecycle owner elements radio; do rg --quiet '^OK$' "{{ output }}/scheme-suite-v2/$suite-tests/module-test.log"; done
    rg --quiet '^OK$' "{{ output }}/scheme-suite-v2/event-tests/event-test.log"
    for runtime in rust scheme; do rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/$runtime-library/library-test.log"; rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/$runtime-consumers/consumer-catalog.log"; rg --quiet '^test result: ok\.' "{{ output }}/$runtime-query/query-scenario.log"; rg --quiet '^native-consumer case=contract_evaluation::contract_org_property_scope_fixture_stays_in_millisecond_budget OK$' "{{ output }}/$runtime-contract/consumer-test.log"; rg --quiet '=+ [0-9]+ passed.*=+$' "{{ output }}/$runtime-python/installed-test.log"; done
    jq --exit-status --arg program "{{ program }}" 'all(.[]; .program == $program)' "{{ output }}/rust-matrix.json"
    jq --exit-status --arg program "{{ program }}" 'all(.[]; .program == $program)' "{{ output }}/scheme-matrix.json"
    just native-runtime-parallel-matrix-check "{{ output }}/rust-matrix.json"
    just native-runtime-parallel-matrix-check "{{ output }}/scheme-matrix.json"
    just native-runtime-qualification-artifact-check "{{ output }}/rust-matrix.json" "{{ output }}/scheme-matrix.json"
    just native-runtime-parallel-compare "{{ output }}/rust-matrix.json" "{{ output }}/scheme-matrix.json"
    just native-document-layout-check

scheme-source-consumer-build output load_path:
    just scheme-lifecycle-value-build "{{ output }}" "{{ load_path }}"

# Link the same standard test suite values ahead of time, not at test startup.
scheme-native-closure-receipt output load_path bridge_source:
    just scheme-native-closure-build "{{ output }}" "{{ load_path }}" "{{ bridge_source }}"
    just scheme-native-closure-test "{{ output }}"
    just scheme-native-closure-failure-control "{{ output }}"

scheme-native-closure-build output load_path bridge_source:
    mkdir -p "{{ output }}"
    just scheme-native-closure-owner-build "{{ output }}" "{{ load_path }}"
    just scheme-native-closure-support-build "{{ output }}" "{{ load_path }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-native-publishing-value-test.ss t/org-native-citation-export-test.ss t/org-native-metadata-value-test.ss t/org-native-source-value-test.ss t/org-native-lifecycle-value-test.ss t/org-native-semantic-owner-test.ss t/org-elements-module-test.ss t/org-radio-match-test.ss t/org-rowan-inline-parser-test.ss t/org-rowan-structural-parser-test.ss t/org-rowan-table-container-parser-test.ss t/org-rowan-event-parser-test.ss t/org-source-headlines-qualification.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-native-closure.ss
    just scheme-native-closure-link "{{ output }}" "{{ load_path }}" "{{ bridge_source }}"

scheme-native-closure-link output load_path bridge_source:
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil "{{ bridge_source }}/scheme/stage-static-runtime.ss" "{{ output }}/lib/orgize/t/org-native-closure.ssi" "{{ output }}" 2>&1 | tee "{{ output }}/native-stage.log"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -V -O -exe -o "{{ output }}/org-native-closure" t/org-native-closure.ss 2>&1 | tee "{{ output }}/native-link.log" | rg '^compile |^invoke |^\.\.\. generate parser artifact|^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

scheme-native-closure-support-build output load_path:
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -V -O languages/org/v1/generated/rowan-event-fixture.ss languages/org/v1/rowan-event-fixture.ss t/org-parser-test-support.ss 2>&1 | tee "{{ output }}/support-build.log" | rg '^compile |^invoke |^\.\.\. generate parser artifact|^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

scheme-native-closure-owner-build output load_path:
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -V -O languages/org/v1/modules/org-elements/catalog.ss languages/org/v1/graph.ss languages/org/v1/modules/org-elements/config.ss languages/org/v1/modules/org-elements/aot.ss languages/org/v1/modules/org-elements/interface.ss languages/org/v1/modules/org-elements/generated/query-source.ss t/org-elements-test-support.ss 2>&1 | tee "{{ output }}/owner-build.log" | rg '^compile |^invoke |^\.\.\. generate parser artifact|^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

scheme-native-closure-test output:
    env -u GERBIL_PATH -u GERBIL_LOADPATH GAMBOPT=max-heap=1G,debug=q python3 tools/ci/watch-real-output.py "{{ output }}/org-native-closure" 2>&1 | tee "{{ output }}/native-test.log"
    test "$(rg -c '^CASE-OK ' '{{ output }}/native-test.log')" -eq 173
    test "$(rg -c '^SUITE-OK ' '{{ output }}/native-test.log')" -eq 12
    rg --quiet '^HARNESS-OK ' "{{ output }}/native-test.log"
    rg --quiet '^OK$' "{{ output }}/native-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/native-test.log"

# A failed standard Gerbil assertion must reach the executable exit status.
scheme-native-closure-failure-control output:
    status=0; env -u GERBIL_PATH -u GERBIL_LOADPATH GAMBOPT=max-heap=1G,debug=q python3 tools/ci/watch-real-output.py "{{ output }}/org-native-closure" --failure-control > "{{ output }}/failure-control.log" 2>&1 || status=$?; test "$status" -eq 42
    rg --quiet '^FAILED$' "{{ output }}/failure-control.log"
    rg --quiet '^ERROR CHECK ' "{{ output }}/failure-control.log"
    ! rg --quiet '^OK$' "{{ output }}/failure-control.log"

# Requalify the whole native semantic suite on the same freshly built owner SDK.
scheme-source-consumer-suite-receipt output load_path:
    just scheme-native-module-fresh-test t/org-native-publishing-value-test.ss "{{ output }}/publishing-tests" "{{ load_path }}"
    just scheme-native-module-fresh-test t/org-native-metadata-value-test.ss "{{ output }}/metadata-tests" "{{ load_path }}"
    just scheme-native-module-fresh-test t/org-native-source-value-test.ss "{{ output }}/source-tests" "{{ load_path }}"
    just scheme-native-module-fresh-test t/org-native-lifecycle-value-test.ss "{{ output }}/lifecycle-tests" "{{ load_path }}"
    just scheme-native-module-fresh-test t/org-native-semantic-owner-test.ss "{{ output }}/owner-tests" "{{ load_path }}"
    just scheme-native-module-fresh-test t/org-elements-module-test.ss "{{ output }}/elements-tests" "{{ load_path }}"
    just scheme-native-module-fresh-test t/org-radio-match-test.ss "{{ output }}/radio-tests" "{{ load_path }}"
    just scheme-event-fresh-test "{{ output }}/event-tests" "{{ load_path }}"

native-lifecycle-value-library-test-receipt binary output:
    just native-value-family-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=native_lifecycle_values_preserve_public_numeric_domains OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_unicode_lowercase_matches_host_reference OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_metadata_time_plans_reject_bad_framing_and_recover OK$' "{{ output }}/library-test.log"
    rg --quiet '^startup-native unit-cases=80 complete OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_clock_windows_cover_leaps_weeks_and_bounds OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_property_tokens_and_progress_cookies_cover_quoting OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_protocol_plans_keep_parameter_presence_and_inert_intent OK$' "{{ output }}/library-test.log"

# One admission for the complete lifecycle/metadata/property/protocol slice.
native-lifecycle-value-admission-check output rust_matrix scheme_matrix:
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__value-funs.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__duration-funs.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__time-funs.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__unicode-context-data.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__unicode-lower-data.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__unicode-funs.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__property-token-funs.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__clock-window-funs.scm"
    test -s "{{ output }}/normal-build-v6/lib/static/orgize__languages__org__v1__modules__org-parser__link-protocol-funs.scm"
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-lifecycle-v6/module-test.log')" -eq 11
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-owner-v6/module-test.log')" -eq 9
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-events-v6/event-test.log')" -eq 103
    test "$(rg -c '^CASE-OK ' '{{ output }}/elements-v6/module-test.log')" -eq 14
    test "$(rg -c '^CASE-OK ' '{{ output }}/radio-v6/module-test.log')" -eq 5
    rg --quiet '^OK$' "{{ output }}/normal-lifecycle-v6/module-test.log"
    rg --quiet '^OK$' "{{ output }}/normal-owner-v6/module-test.log"
    rg --quiet '^OK$' "{{ output }}/normal-events-v6/event-test.log"
    rg --quiet '^OK$' "{{ output }}/elements-v6/module-test.log"
    rg --quiet '^OK$' "{{ output }}/radio-v6/module-test.log"
    rg --quiet '^startup-native unit-cases=80 complete OK$' "{{ output }}/rust-library-v5/library-test.log"
    rg --quiet '^startup-native unit-cases=80 complete OK$' "{{ output }}/scheme-library-v5/library-test.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/rust-consumers-v5/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/scheme-consumers-v5/consumer-catalog.log"
    rg --quiet '^test result: ok\.' "{{ output }}/rust-query-v5/query-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/scheme-query-v5/query-scenario.log"
    rg --quiet '^native-consumer case=lint_contract::lint_contract_org_query_assertion_fixture_stays_in_millisecond_budget OK$' "{{ output }}/rust-contract-v5/consumer-test.log"
    rg --quiet '^native-consumer case=lint_contract::lint_contract_org_query_assertion_fixture_stays_in_millisecond_budget OK$' "{{ output }}/scheme-contract-v5/consumer-test.log"
    ! rg --quiet 'parse_hms|parse_unit_sequence|parse_optional_line_bound|parse_clock_bound_inner|parse_window_block|parse_clock_property_names|split_property_tokens|split_query_parameters|classify_subprotocol|split_uri_protocol' src/semantic_ast/property_model.rs src/semantic_ast/agenda_time.rs src/semantic_ast/runtime_metadata.rs src/semantic_ast/clock_table_time.rs src/semantic_ast/clock_table_properties.rs src/semantic_ast/property_profile.rs src/semantic_ast/link_protocols.rs
    just native-runtime-parallel-matrix-check "{{ rust_matrix }}"
    just native-runtime-parallel-matrix-check "{{ scheme_matrix }}"
    just native-runtime-qualification-artifact-check "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-runtime-parallel-compare "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-document-layout-check

native-value-family-format:
    rustfmt --edition 2024 --config skip_children=true src/org_native_semantic_functions.rs src/org_aot.rs src/org_aot_affiliation.rs src/org_native_events/transport.rs src/semantic_ast/aot_link_resolution.rs src/semantic_ast/lifecycle.rs src/semantic_ast/aot_radio_projection.rs
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/org_native_radio.rs src/semantic_ast/aot_table_projection.rs tests/unit/aot_projection.rs tests/unit/aot_projection/native_value_families.rs benches/support/runtime_corpus.rs build.rs build-support/src/lib.rs

native-value-family-library-test-receipt binary output:
    just native-semantic-owner-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=native_value_families_drive_public_consumers OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_value_batches_handle_ten_thousand_entries OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=native_value_framing_rejects_without_poisoning_owner OK$' "{{ output }}/library-test.log"

# Admit the whole native value-family migration, including both schedulers.
native-value-family-admission-check output rust_matrix scheme_matrix:
    test -s "{{ output }}/normal-build-v2/lib/static/orgize__languages__org__v1__modules__org-parser__family-funs.scm"
    test -s "{{ output }}/normal-build-v2/lib/static/orgize__languages__org__v1__modules__org-elements__radio-match.scm"
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-owner-v2/module-test.log')" -eq 9
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-events-v2/event-test.log')" -eq 103
    test "$(rg -c '^CASE-OK ' '{{ output }}/elements-v2/module-test.log')" -eq 14
    test "$(rg -c '^CASE-OK ' '{{ output }}/radio-v2/module-test.log')" -eq 5
    rg --quiet '^OK$' "{{ output }}/normal-owner-v2/module-test.log"
    rg --quiet '^OK$' "{{ output }}/normal-events-v2/event-test.log"
    rg --quiet '^OK$' "{{ output }}/elements-v2/module-test.log"
    rg --quiet '^OK$' "{{ output }}/radio-v2/module-test.log"
    rg --quiet '^startup-native unit-cases=74 complete OK$' "{{ output }}/rust-library-v4/library-test.log"
    rg --quiet '^startup-native unit-cases=74 complete OK$' "{{ output }}/scheme-library-v4/library-test.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/rust-consumers-v3/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/scheme-consumers-v3/consumer-catalog.log"
    rg --quiet '^test result: ok\.' "{{ output }}/rust-query-v4/query-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/scheme-query-v4/query-scenario.log"
    ! rg --quiet 'include!.*OUT_DIR|define-rust-pure|scheme-pure->rust|org-radio-match-ir-json' src/org_native_semantic_functions.rs tests/integration/org_headline_function_aot.rs languages/org/v1/modules/org-elements
    ! rg --quiet 'write_org_aot_functions|org_aot_functions' build.rs build-support/src
    ! rg --files languages/org/v1/modules/org-elements/generated | rg --quiet '\.ir\.json$'
    just native-runtime-parallel-matrix-check "{{ rust_matrix }}"
    just native-runtime-parallel-matrix-check "{{ scheme_matrix }}"
    just native-runtime-qualification-artifact-check "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-runtime-parallel-compare "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-document-layout-check

# Loadable tests do not interpret generated -S SCM as ordinary source.
scheme-semantic-owner-tape-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss bindings/c/native-runtime-statistics.ss languages/org/v1/rowan-event-tape.ss
    just scheme-native-module-fresh-test t/org-native-semantic-owner-test.ss "{{ output }}" "{{ load_path }}"

native-semantic-owner-format:
    rustfmt --edition 2024 --config skip_children=true src/org_native_events/mod.rs src/org_native_events/transport.rs src/org_aot.rs src/semantic_ast/block_metadata.rs src/semantic_ast/aot_block_projection.rs src/semantic_ast/dynamic_blocks.rs src/semantic_ast/settings.rs src/semantic_ast/prescan.rs src/semantic_ast/aot_projection/document.rs src/semantic_ast/org_contract_evaluation.rs
    rustfmt --edition 2024 --config skip_children=true tests/unit/aot_projection.rs tests/unit/aot_projection/native_owner_plans.rs

native-semantic-owner-library-test-receipt binary output:
    just native-macro-runtime-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=block_lines_and_dynamic_content_are_native OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=keyword_prescan_is_one_native_plan OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=dir_recognition_is_native_and_host_order_is_preserved OK$' "{{ output }}/library-test.log"

native-semantic-owner-dependency-check reference candidate:
    jq --exit-status --slurp 'def helpers: [.modules[] | select(.module | startswith("orgize/languages/org/v1/modules/org-parser/")) | select(.module | test("/(text|block-line|keyword|dir-path)-funs$") | not) | [.module,.scm]] | sort; (.[0] | helpers) as $reference | (.[1] | helpers) as $candidate | ($reference | length > 0) and $reference == $candidate' "{{ reference }}" "{{ candidate }}"

# Admission is one integrated owner gate, not a selected-case smoke result.
native-semantic-owner-admission-check output rust_matrix scheme_matrix:
    test -s "{{ output }}/normal-build-v3/lib/static/orgize__languages__org__v1__modules__org-parser__text-funs.scm"
    test -s "{{ output }}/normal-build-v3/lib/static/orgize__languages__org__v1__modules__org-parser__block-line-funs.scm"
    test -s "{{ output }}/normal-build-v3/lib/static/orgize__languages__org__v1__modules__org-parser__keyword-funs.scm"
    test -s "{{ output }}/normal-build-v3/lib/static/orgize__languages__org__v1__modules__org-parser__dir-path-funs.scm"
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-owner-suite-v3/module-test.log')" -eq 6
    rg --quiet '^OK$' "{{ output }}/normal-owner-suite-v3/module-test.log"
    test "$(rg -c '^CASE-OK ' '{{ output }}/normal-events-v3/event-test.log')" -eq 103
    rg --quiet '^OK$' "{{ output }}/normal-events-v3/event-test.log"
    rg --quiet '^startup-native unit-cases=71 complete OK$' "{{ output }}/rust-library-v2/library-test.log"
    rg --quiet '^startup-native unit-cases=71 complete OK$' "{{ output }}/scheme-library-v2/library-test.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/rust-consumers-v2/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=454 callers=12 complete OK$' "{{ output }}/scheme-consumers-v2/consumer-catalog.log"
    rg --quiet '^test result: ok\.' "{{ output }}/rust-query-v2/query-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/scheme-query-v2/query-scenario.log"
    ! rg --quiet 'org_aot_document_keyword_functions|document-keyword-properties|generate-document-keyword-ir|keyword_option_value.ir' src languages t build-support
    ! just --list | rg --quiet 'generate-document-keyword-ir'
    just native-runtime-parallel-matrix-check "{{ rust_matrix }}"
    just native-runtime-parallel-matrix-check "{{ scheme_matrix }}"
    just native-runtime-qualification-artifact-check "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-runtime-parallel-compare "{{ rust_matrix }}" "{{ scheme_matrix }}"
    just native-document-layout-check

scheme-macro-runtime-normal-build-receipt output load_path:
    mkdir -p "{{ output }}"
    just scheme-parser-build-isolated "{{ output }}" "{{ load_path }}" 12 2>&1 | tee "{{ output }}/normal-build.log"
    test -s "{{ output }}/lib/static/orgize__languages__org__v1__modules__org-parser__event-macro.scm"
    test -s "{{ output }}/lib/static/orgize__languages__org__v1__modules__org-parser__event-source-content.scm"
    test -s "{{ output }}/lib/static/orgize__languages__org__v1__modules__org-parser__macro-funs.scm"

native-macro-runtime-format:
    rustfmt --edition 2024 --config skip_children=true src/org_aot.rs src/semantic_ast/macro_expansion.rs src/semantic_ast/org_contract_evaluation.rs src/org_native_events/mod.rs src/org_native_events/transport.rs tests/unit/aot_projection.rs tests/unit/aot_projection/native_content.rs tests/unit/org_contract_evaluation.rs

native-macro-runtime-library-test-receipt binary output:
    just native-semantic-content-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=public_macro_expansion_and_property_calls_are_native OK$' "{{ output }}/library-test.log"

# Unchanged parser helper owners must come from the qualified semantic closure.
native-macro-runtime-dependency-check reference candidate:
    jq --exit-status --slurp 'def helpers: [.modules[] | select(.module | startswith("orgize/languages/org/v1/modules/org-parser/")) | select(.module != "orgize/languages/org/v1/modules/org-parser/macro-funs") | [.module,.scm]] | sort; (.[0] | helpers) as $reference | (.[1] | helpers) as $candidate | ($reference | length > 0) and $reference == $candidate' "{{ reference }}" "{{ candidate }}"

native-semantic-content-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/aot_table_projection.rs src/semantic_ast/aot_projection/document.rs src/semantic_ast/preprocessing.rs tests/unit/aot_projection.rs tests/unit/aot_projection/native_content.rs

native-semantic-content-library-test-receipt binary output:
    just native-header-consumer-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=public_table_content_and_formula_extents_are_native OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=public_macro_definitions_and_escaped_arguments_are_native OK$' "{{ output }}/library-test.log"

# Only the reviewed source-backed representative graph gains table content fields.
native-semantic-content-snapshot-refresh binary output:
    INSTA_UPDATE=always just native-citation-consumer-case-receipt "{{ binary }}" org_event_aot_parity::representative_fixture_keeps_footnote_ancestry_and_source "{{ output }}"

# Copy a fetched exact revision without touching an existing dirty checkout.
scheme-parser-source-clone source output revision:
    git clone --no-checkout --local "{{ source }}" "{{ output }}"
    git -C "{{ output }}" checkout --detach "{{ revision }}"

# Citation semantics are emitted by the native Scheme scanner, not Rust IR.
scheme-citation-native-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-inline-citation-reference.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-inline-citation.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-inline.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-paragraph.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-strategy.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/rowan-event-runtime.ss

native-citation-projection-format:
    rustfmt --edition 2024 --config skip_children=true src/org_aot.rs src/semantic_ast/aot_projection/mod.rs src/semantic_ast/aot_projection/document.rs src/semantic_ast/aot_projection/citation.rs tests/unit/aot_projection.rs tests/integration/org_citation_aot.rs

scheme-elements-module-fresh-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-elements-module-test.ss
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 5 t/org-elements-module-test.ss 2>&1 | tee "{{ output }}/elements-test.log"
    rg --quiet '^CASE-OK catalog and projected query share one feature interface$' "{{ output }}/elements-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/elements-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/elements-test.log"
    rg --quiet '^OK$' "{{ output }}/elements-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/elements-test.log"

scheme-timestamp-graph-generate output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/grammar.ss languages/org/v1/graph-shape.ss languages/org/v1/graph.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/v1/generate-parser.ss languages/org/v1/generated/parser.rs
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/v1/generate-graph.ss languages/org/v1/generated/graph.rs
    just scheme-parser-dependent-plans-generate "{{ output }}" "{{ output }}/lib:{{ load_path }}"

# Query and Contract packs bind the graph identity and must move with its table.
scheme-parser-dependent-plans-generate output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/v1/modules/org-elements/generate-queries.ss languages/org/v1/modules/org-elements/generated/query-pack.rs
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/v1/modules/org-contract/generate-plan.ss languages/org/v1/modules/org-contract/generated/contract-plan.rs
    just scheme-consumer-packs-regenerate "{{ output }}" "{{ load_path }}"

native-timestamp-projection-format:
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/aot_timestamp_projection.rs src/semantic_ast/timestamp_model.rs tests/unit/aot_projection.rs tests/unit/contract_feature.rs

scheme-list-opaque-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-list.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/modules/org-parser/event-strategy.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/rowan-event-runtime.ss

scheme-structural-event-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-rowan-structural-parser-test.ss
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 3 t/org-rowan-structural-parser-test.ss 2>&1 | tee "{{ output }}/structural-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/structural-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/structural-test.log"
    rg --quiet '^OK$' "{{ output }}/structural-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/structural-test.log"

scheme-rowan-event-fixture-generate output load_path destination:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/rowan-event-fixture.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/v1/generate-rowan-event-fixture.ss "{{ destination }}"

native-rowan-fixture-format:
    rustfmt --edition 2024 --config skip_children=true tests/integration/org_rowan_event_handoff.rs

native-rowan-fixture-test-receipt binary output:
    mkdir -p "{{ output }}"
    python3 tools/ci/watch-real-output.py "{{ binary }}" explicit_startup_precedes_parallel_org_rowan_event_handoff_cases --exact --nocapture 2>&1 | tee "{{ output }}/rowan-fixture-test.log"
    rg --quiet '^startup-native suite=org_rowan_event_handoff concurrent-cases=23 complete OK$' "{{ output }}/rowan-fixture-test.log"
    rg --quiet '^test result: ok\. 1 passed; 0 failed;' "{{ output }}/rowan-fixture-test.log"
    ! rg --quiet '^FAIL:|^error:|panicked at|^test result: FAILED' "{{ output }}/rowan-fixture-test.log"

# Fresh child suites are required when editing grammar expectations. The caller
# supplies the separately qualified parser overlay; this is not an AOT link gate.
scheme-event-fresh-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/v1/generated/rowan-event-fixture.ss languages/org/v1/rowan-event-fixture.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-rowan-inline-parser-test.ss t/org-rowan-structural-parser-test.ss t/org-rowan-table-container-parser-test.ss t/org-rowan-event-parser-test.ss
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 5 t/org-rowan-event-parser-test.ss 2>&1 | tee "{{ output }}/event-test.log"
    rg --quiet '^CASE timestamp cookie admission is native Scheme, including recovery' "{{ output }}/event-test.log"
    rg --quiet '^CASE-OK timestamp cookie admission is native Scheme, including recovery$' "{{ output }}/event-test.log"
    rg --quiet '^CASE-OK citation header components are native Scheme source spans$' "{{ output }}/event-test.log"
    rg --quiet '^CASE-OK citation affix content ranges are native Scheme decisions$' "{{ output }}/event-test.log"
    rg --quiet '^SUITE Org inline source-backed Objects$' "{{ output }}/event-test.log"
    rg --quiet '^SUITE Org structural Elements and metadata$' "{{ output }}/event-test.log"
    rg --quiet '^SUITE Org tables containers and AOT receipts$' "{{ output }}/event-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/event-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/event-test.log"
    rg --quiet '^OK$' "{{ output }}/event-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/event-test.log"

scheme-event-tape-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss bindings/c/native-runtime-statistics.ss
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ justfile_directory() }}/target/gerbil-test/lib:{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test -v 3 t/org-event-tape-test.ss 2>&1 | tee target/gerbil-test/org-event-tape.log
    rg --quiet '^MODULE-OK' target/gerbil-test/org-event-tape.log
    rg --quiet '^HARNESS-OK' target/gerbil-test/org-event-tape.log
    rg --quiet '^OK$' target/gerbil-test/org-event-tape.log
    ! rg --quiet 'ERROR|FAILED|FAILURE' target/gerbil-test/org-event-tape.log

# Native actor lifecycle admission, independent of foreign-thread attachment.
scheme-native-actor-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test -v 3 t/org-native-actor-test.ss

scheme-native-actor-build parser_lib poo_flow_lib:
    mkdir -p target/gerbil-parser
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil compile -O -S bindings/c/orgize-actor.ss

# Compile the actual FFI export with Gerbil's optimizer; do not patch Gambit.
scheme-parser-bridge parser_lib poo_flow_lib bridge_lib:
    mkdir -p target/gerbil-parser
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}:{{ bridge_lib }}" gerbil compile -O -S bindings/c/native-clock.ss
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}:{{ bridge_lib }}" gerbil compile -O -S -cc-options '-Ibindings/c/include' languages/org/v1/modules/org-contract/grammar.ss languages/org/v1/modules/org-contract/parser.ss languages/org/v1/modules/org-contract/contract-normalize.ss languages/org/v1/modules/org-contract/expectation-grammar.ss languages/org/v1/modules/org-contract/expectation-parser.ss languages/org/v1/rowan-event-runtime.ss languages/org/v1/rowan-event-tape.ss bindings/c/orgize-scheme-runtime.ss bindings/c/orgize-parser.ss

generate-expectation-parser parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi languages/org/v1/modules/org-contract/generate-expectation-parser.ss languages/org/v1/modules/org-contract/generated/expectation.rs

scheme-parser-stage parser_lib poo_flow_lib bridge_lib: (scheme-parser-bridge parser_lib poo_flow_lib bridge_lib)
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}:{{ bridge_lib }}" gxi bindings/c/stage-parser.ss target/gerbil-parser

# After the full stage has compiled grammar/parser owners, refresh only the
# private ABI projector. Native packaging still fingerprints every input.
scheme-parser-tape-stage parser_lib poo_flow_lib bridge_lib:
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ justfile_directory() }}/target/gerbil-parser/lib:{{ justfile_directory() }}/.gerbil/lib:{{ parser_lib }}:{{ poo_flow_lib }}:{{ bridge_lib }}" gerbil compile -O -S bindings/c/native-clock.ss languages/org/v1/modules/org-contract/contract-normalize.ss languages/org/v1/rowan-event-tape.ss
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ justfile_directory() }}/.gerbil/lib:{{ parser_lib }}:{{ poo_flow_lib }}:{{ bridge_lib }}" gxi bindings/c/stage-parser.ss target/gerbil-parser

# Explicit source-checkout qualification; the package's existing build.ss owns
# its graph. This does not install/uninstall packages or guess sibling paths.
scheme-parser-package source load_path:
    cd "{{ source }}" && {{ scheme_env }} GERBIL_BUILD_CORES=1 GERBIL_BUILD_VERBOSE=1 GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="$PWD/.gerbil" GERBIL_LOADPATH="{{ load_path }}" gxi build.ss compile --optimized

# Rebased engine qualification must not reuse an older compiled parser graph.
scheme-parser-package-isolated source output load_path:
    mkdir -p "{{ output }}"
    cd "{{ source }}" && {{ scheme_env }} GERBIL_BUILD_VERBOSE=1 GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build.ss compile --optimized

# Exercise upstream scanner/LR/recovery changes after an exact-head rebase.
scheme-parser-engine-fixtures source output load_path:
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/fixtures/lr1-construction.ss t/benchmarks/contextual-scanner/parser-fixture.ss

scheme-parser-engine-test source output load_path:
    cd "{{ source }}" && GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 3 t/contextual-scanner-test.ss t/contextual-dispatch-test.ss t/contextual-parser-test.ss t/contextual-deferred-scanner-test.ss t/canonical-lr1-reference-test.ss t/recovery-incremental-test.ss 2>&1 | tee "{{ output }}/engine-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/engine-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/engine-test.log"
    rg --quiet '^OK$' "{{ output }}/engine-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/engine-test.log"

# Preserve the exact compiled dependency closure for a matched algorithm A/B.
scheme-parser-lib-save parser_lib label:
    test ! -e "target/scheme-receipt-libs/{{ label }}"
    mkdir -p target/scheme-receipt-libs
    cp -R "{{ parser_lib }}" "target/scheme-receipt-libs/{{ label }}"

# Preserve tracked and untracked parser work before moving its exact base.
# Restore with apply, not pop: the recovery object remains available.
scheme-parser-rebase-save source label:
    cd "{{ source }}" && git stash push --include-untracked --message "{{ label }}"

scheme-parser-rebase-onto source revision:
    cd "{{ source }}" && git rebase "{{ revision }}"

scheme-parser-rebase-restore source stash:
    cd "{{ source }}" && git stash apply --index "{{ stash }}"

scheme-event-fold-build source load_path:
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="$PWD/.gerbil" GERBIL_LOADPATH="$PWD/.gerbil/lib:{{ load_path }}" gerbil compile -O src/compiler/event-fold-runtime.ss

# Isolate an algorithm experiment from the currently linked native program.
scheme-event-fold-candidate-build source output load_path:
    mkdir -p "{{ output }}"
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O src/compiler/event-source-lines.ss src/compiler/event-fold-runtime.ss src/compiler/event-strategy-aot.ss

# Overlay only the preserved fold payload, never an older scanner/parser graph.
scheme-event-fold-module-save parser_lib output:
    test ! -e "{{ output }}"
    mkdir -p "{{ output }}/gerbil-parser/src/compiler"
    cp "{{ parser_lib }}"/gerbil-parser/src/compiler/event-fold-runtime* "{{ output }}/gerbil-parser/src/compiler/"

# Generate the same candidate's static modules for the native AOT owner.
scheme-event-fold-candidate-static source output load_path:
    mkdir -p "{{ output }}"
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O -S src/compiler/event-source-lines.ss src/compiler/event-fold-runtime.ss src/compiler/event-strategy-aot.ss

scheme-event-fold-state-test source output load_path:
    mkdir -p "{{ output }}"
    cd "{{ source }}" && GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 5 t/event-fold-state-sharing-test.ss 2>&1 | tee "{{ output }}/state-test.log"
    rg --quiet '^CASE-OK ' "{{ output }}/state-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/state-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/state-test.log"
    rg --quiet '^OK$' "{{ output }}/state-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/state-test.log"

# Replays the rejected candidate's private tests against an explicit overlay,
# not against the current parser source (which no longer exposes its helper).
scheme-event-fold-source-span-experiment-test test_source output load_path:
    mkdir -p "{{ output }}"
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ load_path }}" gerbil test -v 3 "{{ test_source }}" 2>&1 | tee "{{ output }}/source-span-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/source-span-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/source-span-test.log"
    rg --quiet '^OK$' "{{ output }}/source-span-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/source-span-test.log"

scheme-event-fold-candidate-test source output load_path:
    just scheme-native-module-fresh-test "{{ source }}/t/event-fold-test.ss" "{{ output }}" "{{ load_path }}"

scheme-parser-grammar-facade source load_path:
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="$PWD/.gerbil" GERBIL_LOADPATH="{{ load_path }}" gerbil compile -O -S language-support.ss

scheme-parser-build load_path cores="12":
    just scheme-parser-build-isolated "{{ justfile_directory() }}/target/gerbil-parser" "{{ load_path }}" "{{ cores }}"

# Requalify the complete Org graph without retaining a previous engine's IR.
scheme-parser-build-isolated output load_path cores="12":
    mkdir -p "{{ output }}"
    {{ scheme_env }} GERBIL_BUILD_CORES="{{ cores }}" GERBIL_BUILD_VERBOSE=1 GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build.ss compile --optimized
    {{ scheme_env }} GERBIL_BUILD_CORES="{{ cores }}" GERBIL_BUILD_VERBOSE=1 GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build-parser.ss compile --optimized

scheme-parser-stage-isolated output load_path:
    just scheme-parser-batch-stage-isolated "{{ output }}" "{{ load_path }}"

# Refresh the private bounded batch projector in a separate compiler-owned
# program. Its unchanged dependency closure comes from explicit compiled libs.
scheme-parser-batch-stage-isolated output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O -S -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss bindings/c/native-runtime-statistics.ss languages/org/v1/rowan-event-tape.ss
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive bindings/c/stage-parser.ss "{{ output }}"

native-runtime-batch-format:
    rustfmt --edition 2024 --config skip_children=true src/org_native_events/mod.rs src/org_native_events/transport.rs src/org_native_events/batch.rs src/org_aot.rs src/document/elements.rs src/document/org_elements_aot.rs tests/unit/org_native_batch.rs tests/unit/org_native_batch_wire.rs tests/unit/document_org_elements_aot.rs tests/unit/document_org_elements_query_project.rs tests/unit/lib.rs

# Separate same-program ASP legs: a failed gate never hides the other leg.
native-runtime-query-batch-run binary:
    "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate --list | rg 'document_query_org_elements_aot_stays_inside_scenario_gate: test$'
    "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate --ignored --nocapture --test-threads=1

native-runtime-query-individual-control-run binary:
    "{{ binary }}" document_query_org_elements_aot_individual_control --list | rg 'document_query_org_elements_aot_individual_control: test$'
    "{{ binary }}" document_query_org_elements_aot_individual_control --ignored --nocapture --test-threads=1

# Select a qualified compiler-owned closure without rewriting its module paths.
# Keep the previous default manifest recoverable; its old modules are untouched.
native-parser-program-select manifest previous_label:
    jq --exit-status '.schema == "gerbil-scheme-rust.aot-program.v1" and (.modules | length > 0) and (.stub | type == "string")' "{{ manifest }}"
    mkdir -p target/native-program-history target/gerbil-parser
    if test -f target/gerbil-parser/program.json; then cp -n target/gerbil-parser/program.json "target/native-program-history/{{ previous_label }}.json"; fi
    cp "{{ manifest }}" target/gerbil-parser/program.json
    cmp "{{ manifest }}" target/gerbil-parser/program.json

# Contract's existing graph-query dependency needs static Scheme artifacts too.
scheme-contract-deps source load_path:
    cd "{{ source }}" && GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="$PWD/.gerbil" GERBIL_LOADPATH="{{ load_path }}" gerbil compile -O -S src/modules/parser/graph-query-types.ss src/modules/parser/graph-query-objects.ss src/modules/parser/graph-query-funs.ss graph-query-support.ss

# Consumer fixtures are generated by the public Scheme AOT APIs, not by
# replacing digest strings in Rust or disabling graph identity admission.
scheme-consumer-packs-regenerate output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive tests/fixtures/org-elements/generate-customer-queries.ss tests/fixtures/org-elements/generated/customer-query-pack.rs
    {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive tests/fixtures/org-contract/generate-customer-contracts.ss tests/fixtures/org-contract/generated/customer-contract-pack.rs

# Gerbil supplies the program manifest; its existing Rust build owner links it.
native-parser-test manifest gsc:
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline -vv --test gerbil_rowan_cutover -- --nocapture

native-build-test bridge_source filter="native_command":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" cargo test --offline --manifest-path "{{ bridge_source }}/Cargo.toml" -p gerbil-scheme-native-build --lib "{{ filter }}" -- --nocapture

native-owner-program-build source:
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="$PWD/.gerbil" gerbil compile -O scheme/program-build.ss

native-owner-lifecycle-check source gsc:
    mkdir -p target/runtime-policy
    {{ native_env }} cc -O2 -DGERBIL_SCHEME_RUST_EXTERNAL_PROGRAM -I"{{ parent_directory(parent_directory(gsc)) }}/include" -c "{{ source }}/native/runtime.c" -o target/runtime-policy/lifecycle.o

# Explicit checkout override while maintaining the native-build owner's PR.
native-parser-test-owner manifest gsc bridge_source test="gerbil_rowan_cutover" runtime="runtime-rust":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --test "{{ test }}" -- --nocapture

native-parser-check-owner manifest gsc bridge_source:
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo check --offline -vv --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --tests --benches

native-parser-unit-owner manifest gsc bridge_source:
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --lib org_aot::native_events::tests -- --nocapture

# Measure the existing public-entry Criterion scenarios, not a new harness.
native-parser-bench-owner manifest gsc bridge_source profile="release" filter='Org::parse/(doc|plain-links|quote-heavy)\.org':
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo bench --offline --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --bench parse -- "{{ filter }}" --sample-size 10 --warm-up-time 0.2 --measurement-time 1 --noplot

# Reuse Criterion's committed input files and ASP's Scheme timing/memory owner.
# Matched execution-owner lanes; no parser/backend substitution.
native-runtime-bench-owner manifest gsc bridge_source runtime="runtime-rust" profile="release":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo bench --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --bench org_runtime -- --sample-size 10 --warm-up-time 0.2 --measurement-time 1 --noplot

native-runtime-cold-owner manifest gsc bridge_source runtime="runtime-rust" profile="release" samples="10":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo run --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --example runtime_cold -- "{{ samples }}"

# Explicit local Apple ld recursion diagnostic. Only the selected final target
# changes linker implementation; profiles, Thin LTO and dead-strip stay intact.
# Cargo also builds CLI targets for benches/tests, so the explicit diagnostic
# covers the whole Apple target invocation, not only the final selected target.
# This is not a production/runtime fallback, nor a portable package receipt.
native-runtime-link-owner manifest gsc bridge_source runtime kind name profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo rustc --offline -vv --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' "{{ kind }}" "{{ name }}"

# Build the explicit-startup integration suites; execution remains separately bounded.
native-runtime-integration-build-owner manifest gsc bridge_source runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --no-run --test gerbil_rowan_cutover --test native_host_children --test org_cutover_parity --test org_rowan_event_handoff --test org_aot_owned_projection

native-runtime-bench-run binary baseline="base":
    "{{ binary }}" --bench --sample-size 10 --warm-up-time 0.2 --measurement-time 1 --noplot --save-baseline "{{ baseline }}"

# Preserve an already-qualified executable before Cargo replaces its artifact.
native-runtime-binary-save binary label:
    mkdir -p target/runtime-receipt-binaries
    cp -n "{{ binary }}" "target/runtime-receipt-binaries/{{ label }}"

# Exactly N distinct documents per batch; does not use Criterion calibration.
native-runtime-corpus-run binary output documents="1000,10000" callers="1,8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_DOCS="{{ documents }}" ORGIZE_RUNTIME_CORPUS_CALLERS="{{ callers }}" ORGIZE_RUNTIME_CORPUS_REPEATS="{{ repeats }}" ORGIZE_RUNTIME_CORPUS_OUTPUT="{{ output }}" "{{ binary }}" --bench --noplot

native-runtime-corpus-compare rust_receipt scheme_receipt:
    jq --exit-status --slurp --from-file benches/support/runtime_corpus_compare.jq "{{ rust_receipt }}" "{{ scheme_receipt }}"

# Full scale is four distinct batches, not a small smoke mistaken for closure.
native-runtime-corpus-matrix-check receipt:
    jq --exit-status 'length == 4 and ((map([.documents, .callers, .repetition]) | sort) == [[1000,1,1],[1000,8,1],[10000,1,1],[10000,8,1]]) and all(.[]; .documents == .completed and .documents == .distinct_documents and .driver == "tokio") and ((map(.completed) | add) == 22000)' "{{ receipt }}"

native-runtime-tokio-corpus-run binary output documents="1000,10000" callers="1,8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_DRIVER=tokio just native-runtime-corpus-run "{{ binary }}" "{{ output }}" "{{ documents }}" "{{ callers }}" "{{ repeats }}"

# High-concurrency admissions plus independent native execution domains.
# The supervisor never initializes Gerbil; workers exec and warm up before timing.
native-runtime-parallel-corpus-run binary output documents="1000,10000" callers="64" domains="1,4,8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_DOMAINS="{{ domains }}" just native-runtime-corpus-run "{{ binary }}" "{{ output }}" "{{ documents }}" "{{ callers }}" "{{ repeats }}"

native-runtime-parallel-build-owner manifest gsc bridge_source runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo bench --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --bench org_runtime --no-run

native-runtime-parallel-matrix-check receipt:
    jq --exit-status 'length == 6 and ((map([.documents,.callers,.domains,.repetition]) | sort) == [[1000,64,1,1],[1000,64,4,1],[1000,64,8,1],[10000,64,1,1],[10000,64,4,1],[10000,64,8,1]]) and all(.[]; (.schema == "orgize.runtime-parallel.v1" or .schema == "orgize.runtime-parallel.v2" and .mode == "qualification") and .documents == .completed and .documents == .distinct_documents and .max_in_flight == .callers and .max_active_worker_requests == .domains and (.worker_pids | unique | length) == .domains and (.worker_completed | add) == .completed and all(.worker_completed[]; . > 0) and .worker_exit_success) and ((map(.completed) | add) == 33000)' "{{ receipt }}"

native-runtime-parallel-compare rust_receipt scheme_receipt:
    jq --exit-status --slurp --from-file benches/support/runtime_parallel_compare.jq "{{ rust_receipt }}" "{{ scheme_receipt }}"

native-runtime-parallel-format:
    rustfmt --edition 2024 benches/org_runtime.rs benches/support/runtime_corpus.rs benches/support/runtime_parallel.rs

native-runtime-parallel-report receipt:
    jq '[.[] | {backend, documents, callers, domains, completed, max_in_flight, max_active_worker_requests, documents_per_second, end_to_end_p95_ms: (.end_to_end_latency_ns.p95 / 1000000), end_to_end_p99_ms: (.end_to_end_latency_ns.p99 / 1000000)}]' "{{ receipt }}"

native-runtime-performance-run binary output documents="1000,10000" callers="64" domains="8" repeats="1":
    ORGIZE_RUNTIME_CORPUS_MODE=performance just native-runtime-parallel-corpus-run "{{ binary }}" "{{ output }}" "{{ documents }}" "{{ callers }}" "{{ domains }}" "{{ repeats }}"

native-runtime-stage-report receipt:
    jq '[.[] | {backend,mode,documents,callers,domains,profile_enabled,documents_per_second,stage_mean_ms:(.stage_timings_ns | map_values(.mean / 1000000))}]' "{{ receipt }}"

native-runtime-owner-stage-report receipt:
    jq --exit-status --from-file benches/support/runtime_owner_stage_report.jq "{{ receipt }}"

native-runtime-profile-format:
    rustfmt --edition 2024 --config skip_children=true src/runtime_profile/mod.rs src/runtime_profile/collector.rs src/runtime_profile/dispatch.rs src/runtime_profile/transport.rs src/runtime_profile/wire.rs src/org_native_events/mod.rs src/org_native_events/transport.rs src/org_aot.rs tests/unit/runtime_profile.rs

native-runtime-transport-stage-scale-check receipt: (native-runtime-inner-stage-scale-check receipt)
    jq --exit-status 'all(.[]; .stage_timings_ns | has("native.owner_admission") and has("native.owner_service_inclusive") and has("native.result_copy") and has("native.completion_handoff"))' "{{ receipt }}"
    just native-runtime-transport-timing-check "{{ receipt }}"

native-runtime-transport-timing-check receipt:
    jq --exit-status 'length > 0 and all(.[]; .stage_timings_ns as $stages | all(["native.owner_admission","native.owner_service_inclusive","native.result_copy","native.completion_handoff"][]; $stages[.].total > 0 and $stages[.].mean > 0))' "{{ receipt }}"

native-runtime-transport-repeat-scale-check receipt: (native-runtime-profile-receipt-check receipt)
    jq --exit-status 'length == 4 and (map([.documents,.callers,.domains,.repetition]) | sort) == [[1000,64,8,1],[1000,64,8,2],[10000,64,8,1],[10000,64,8,2]] and (map(.completed) | add) == 22000 and all(.[]; .stage_timings_ns | has("native.scheme_fold") and has("native.tape_encode") and has("native.scheme_thread_cpu") and has("native.owner_admission") and has("native.owner_service_inclusive") and has("native.result_copy") and has("native.completion_handoff"))' "{{ receipt }}"
    just native-runtime-transport-timing-check "{{ receipt }}"

native-runtime-profile-receipt-check receipt:
    jq --exit-status 'length > 0 and all(.[]; .schema == "orgize.runtime-parallel.v2" and .profile_enabled and .completed == .documents and .distinct_documents == .documents and .max_in_flight == .callers and .max_active_worker_requests == .domains and .worker_exit_success and (.worker_pids | unique | length) == .domains and (.worker_completed | add) == .completed and (.mode == "performance" and .artifact == null or .mode == "qualification" and (.artifact | type) == "string") and (.stage_timings_ns | has("native.transport_inclusive") and has("rowan.build") and has("graph.project") and has("supervisor.queue")))' "{{ receipt }}"

native-runtime-profile-scale-check receipt: (native-runtime-profile-receipt-check receipt)
    jq --exit-status 'length == 2 and (map([.documents,.callers,.domains,.repetition]) | sort) == [[1000,64,8,1],[10000,64,8,1]] and (map(.completed) | add) == 11000' "{{ receipt }}"

native-runtime-inner-stage-scale-check receipt: (native-runtime-profile-scale-check receipt)
    jq --exit-status 'all(.[]; .stage_timings_ns | has("native.scheme_fold") and has("native.tape_encode") and has("native.scheme_thread_cpu"))' "{{ receipt }}"

# Semantic equality across an explicitly changed native program is distinct
# from the strict same-program artifact check and the runtime comparator.
native-runtime-qualification-semantic-check reference candidate:
    jq --exit-status --slurp '.[0] as $reference | (.[1] | length > 0) and all(.[1][]; . as $candidate | .mode == "qualification" and .completed == .documents and (.artifact | type) == "string" and any($reference[]; .documents == $candidate.documents and .corpus == $candidate.corpus and .artifact == $candidate.artifact))' "{{ reference }}" "{{ candidate }}"

native-runtime-qualification-artifact-check reference candidate:
    jq --exit-status --slurp '.[0] as $reference | all(.[1][]; . as $candidate | (.artifact | type) == "string" and any($reference[]; .documents == $candidate.documents and .corpus == $candidate.corpus and .program == $candidate.program and .artifact == $candidate.artifact))' "{{ reference }}" "{{ candidate }}"

# Retest an immutable executable; this gate is not a cross-program comparator.
native-runtime-control-identity-check reference candidate: (native-runtime-profile-receipt-check candidate)
    jq --exit-status --slurp '.[0] as $reference | (.[1] | length > 0) and all(.[1][]; . as $candidate | any($reference[]; .backend == $candidate.backend and .mode == $candidate.mode and .program == $candidate.program and .adapter == $candidate.adapter and .benchmark == $candidate.benchmark and .corpus == $candidate.corpus and .documents == $candidate.documents and .callers == $candidate.callers and .domains == $candidate.domains and .repetition == $candidate.repetition))' "{{ reference }}" "{{ candidate }}"

native-runtime-test-run binary filter="":
    "{{ binary }}" "{{ filter }}" --nocapture

native-runtime-consumer-test-run binary case="":
    ORGIZE_NATIVE_CONSUMER_CASE="{{ case }}" just native-runtime-test-run "{{ binary }}"

# Empty selection retains the fixture's exact original-catalog assertion.
native-runtime-consumer-catalog-receipt binary output:
    mkdir -p "{{ output }}"
    just native-runtime-consumer-test-run "{{ binary }}" "" 2>&1 | tee "{{ output }}/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=[1-9][0-9]* callers=[1-9][0-9]* complete OK$' "{{ output }}/consumer-catalog.log"
    rg --quiet '^test result: ok\.' "{{ output }}/consumer-catalog.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/consumer-catalog.log"

# Same libtest binary in two processes: exclusive native startup must not race
# the pure/policy tests' stdio or host work. All 54 native cases stay concurrent.
native-runtime-library-test-run binary:
    "{{ binary }}" tests::explicit_startup_precedes_parallel_native_unit_cases --exact --nocapture
    "{{ binary }}" --skip tests::explicit_startup_precedes_parallel_native_unit_cases --nocapture

native-timestamp-library-test-receipt binary output:
    mkdir -p "{{ output }}"
    just native-runtime-library-test-run "{{ binary }}" 2>&1 | tee "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=public_timestamp_date_fields_are_classified_by_scheme OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=public_timestamp_cookie_fields_are_classified_by_scheme OK$' "{{ output }}/library-test.log"
    rg --quiet '^test contract_feature::tests::generated_consumers_bind_the_current_graph_identity \.\.\. ok$' "{{ output }}/library-test.log"
    rg --quiet '^test result: ok\.' "{{ output }}/library-test.log"
    ! rg --quiet 'FAILED|test result: FAILED|panicked at' "{{ output }}/library-test.log"

native-citation-library-test-receipt binary output:
    just native-timestamp-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=public_citation_header_fields_are_classified_by_scheme OK$' "{{ output }}/library-test.log"
    rg --quiet '^native-unit case=public_citation_affix_ranges_are_classified_by_scheme OK$' "{{ output }}/library-test.log"

native-source-switch-library-test-receipt binary output:
    just native-timestamp-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=public_source_switch_arguments_are_classified_by_scheme OK$' "{{ output }}/library-test.log"

native-source-header-library-test-receipt binary output:
    just native-source-switch-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=public_header_argument_structure_is_owned_by_scheme OK$' "{{ output }}/library-test.log"

native-header-consumer-library-test-receipt binary output:
    just native-source-header-library-test-receipt "{{ binary }}" "{{ output }}"
    rg --quiet '^native-unit case=public_keyword_and_include_headers_are_decoded_from_native_content OK$' "{{ output }}/library-test.log"

# Refresh only the reviewed graph snapshots affected by native header fields.
native-source-header-snapshot-refresh binary output:
    INSTA_UPDATE=always just native-citation-consumer-case-receipt "{{ binary }}" org_event_aot_parity::tracked_org_fixtures_keep_source_and_element_ancestry "{{ output }}"

# Keep the existing explicit-startup consumer fixture and exact catalog lookup.
native-citation-consumer-case-receipt binary case output:
    mkdir -p "{{ output }}"
    just native-runtime-consumer-test-run "{{ binary }}" "{{ case }}" 2>&1 | tee "{{ output }}/consumer-test.log"
    rg --quiet '^native-consumer case={{ case }} OK$' "{{ output }}/consumer-test.log"
    rg --quiet '^test result: ok\.' "{{ output }}/consumer-test.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/consumer-test.log"

native-citation-consumer-test-receipt binary output:
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::malformed_citation_marker_reaches_the_owned_diagnostic "{{ output }}/malformed"
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::scheme_citation_header_aot_projects_style_and_variant "{{ output }}/header"
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::public_ast_projects_scheme_citation_reference_fields "{{ output }}/typed"
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::scheme_declared_citations_project_into_rowan_and_graph "{{ output }}/recovery"
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::citation_references_keep_distinct_keys_and_source_fields "{{ output }}/references"
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::citation_global_prefix_and_suffix_project_from_scheme "{{ output }}/affixes"
    just native-citation-consumer-case-receipt "{{ binary }}" org_citation_aot::citation_affix_graph_keeps_source_ranges_and_nested_objects "{{ output }}/rich-affixes"

native-runtime-query-scenario-receipt binary output:
    mkdir -p "{{ output }}"
    "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate --list | rg '^document::org_elements_query_project_tests::document_query_org_elements_aot_stays_inside_scenario_gate: test$'
    just native-runtime-ignored-test-run "{{ binary }}" document_query_org_elements_aot_stays_inside_scenario_gate 2>&1 | tee "{{ output }}/query-scenario.log"
    rg --quiet '^test result: ok\.' "{{ output }}/query-scenario.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/query-scenario.log"

# Qualify the linked parser without depending on the checkout as the cwd.
native-runtime-library-detached-test-run binary directory:
    cd "{{ directory }}" && "{{ binary }}" tests::explicit_startup_precedes_parallel_native_unit_cases --exact --nocapture

# Existing ignored performance scenarios retain their original limits.
native-runtime-ignored-test-run binary filter:
    "{{ binary }}" "{{ filter }}" --ignored --nocapture

# Reject an unprofiled binary instead of accepting libtest's zero-test success.
native-runtime-query-profile-run binary:
    "{{ binary }}" document_query_org_elements_aot_profiles_existing_scenario_gate --list | rg 'document_query_org_elements_aot_profiles_existing_scenario_gate: test$'
    "{{ binary }}" document_query_org_elements_aot_profiles_existing_scenario_gate --ignored --nocapture --test-threads=1

native-runtime-query-profile-format:
    rustfmt --edition 2024 --config skip_children=true tests/unit/document_org_elements_query_project.rs

# Real project Query; count is explicit and the libtest inventory is checked.
# Qualification has no invented large-project latency pass threshold.
native-runtime-query-scale-run binary documents:
    "{{ binary }}" document_query_native_parallel_scale_qualification --list | rg 'document_query_native_parallel_scale_qualification: test$'
    ORGIZE_QUERY_WORKER_TRACE=1 ORGIZE_QUERY_SCALE_DOCUMENTS="{{ documents }}" "{{ binary }}" document_query_native_parallel_scale_qualification --ignored --nocapture --test-threads=1

native-runtime-query-worker-format:
    rustfmt --edition 2024 --config skip_children=true src/runtime_profile/mod.rs src/runtime_profile/worker.rs src/document/elements.rs src/document/mod.rs tests/unit/runtime_profile.rs tests/unit/document_query_scale.rs

# Supervisor never initializes native runtime; each case is a fresh child.
native-runtime-lifecycle-run binary:
    "{{ binary }}"

native-runtime-cold-run binary samples="10":
    "{{ binary }}" "{{ samples }}"

native-runtime-check-owner manifest gsc bridge_source runtime="runtime-rust" profile="dev":
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo check --offline --profile "{{ profile }}" --no-default-features --features "{{ runtime }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --tests --benches --examples

native-parser-dependency-update bridge_source:
    cargo update --package gerbil-parser-rowan --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"'

native-runtime-consumer-build-owner manifest gsc bridge_source runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --no-run --lib --tests

native-runtime-library-build-owner manifest gsc bridge_source runtime="runtime-rust" profile="release":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --no-run --lib -vv

native-runtime-library-build-receipt manifest gsc bridge_source output runtime="runtime-rust" profile="release":
    mkdir -p "{{ output }}"
    just native-runtime-library-build-owner "{{ manifest }}" "{{ gsc }}" "{{ bridge_source }}" "{{ runtime }}" "{{ profile }}" 2>&1 | tee "{{ output }}/library-build.log"
    rg --quiet '^    Finished ' "{{ output }}/library-build.log"
    test "$(rg -c '^  Executable ' "{{ output }}/library-build.log")" -eq 1
    ! rg --quiet '^error:|^error\[' "{{ output }}/library-build.log"

# Build one matched candidate for native units, the full consumer catalog and
# scale receipts, including the CLI dependency used by integration consumers.
native-runtime-closure-build-owner manifest gsc bridge_source runtime="runtime-rust" profile="release" include_bench="true":
    {{ native_env }} {{ native_rust_flags }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" cargo test --offline --no-default-features --features "{{ runtime }}" --profile "{{ profile }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' --no-run --lib --test integration_test {{ if include_bench == "true" { "--bench org_runtime" } else { "" } }} -vv

native-semantic-wrapper-format:
    rustfmt --edition 2024 --config skip_children=true src/org_native_semantic_functions.rs tests/integration/support/native_semantic_values.rs tests/integration/org_headline_function_aot.rs

# Reject the audited production-library warning group, not dependency/linker warnings.
native-semantic-wrapper-warning-check output:
    rg --quiet 'Finished .*release.*profile' "{{ output }}/release-build.log"
    ! rg --quiet '^warning: `orgize` \(lib\) generated' "{{ output }}/release-build.log"
    ! rg --quiet '^[[:space:]]*--> (src/org_native_semantic_functions.rs|tests/integration/(org_headline_function_aot.rs|support/native_semantic_values.rs)):' "{{ output }}/release-build.log"
    ! rg --quiet '^error:|^error\[' "{{ output }}/release-build.log"

# Both feature builds and their unchanged explicit-startup consumer controls.
native-semantic-wrapper-warning-negative-check previous:
    rg --quiet '^warning: `orgize` \(lib\) generated 30 warnings' "{{ previous }}/release-build.log"
    ! just native-semantic-wrapper-warning-check "{{ previous }}"

native-semantic-wrapper-admission-check output:
    just native-semantic-wrapper-warning-check "{{ output }}/rust-build"
    just native-semantic-wrapper-warning-check "{{ output }}/scheme-build"
    rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/rust/library-test.log"
    rg --quiet '^startup-native unit-cases=84 complete OK$' "{{ output }}/scheme/library-test.log"
    rg --quiet '^test result: ok\. 24 passed; 0 failed; 2 ignored;' "{{ output }}/rust/library-test.log"
    rg --quiet '^test result: ok\. 24 passed; 0 failed; 2 ignored;' "{{ output }}/scheme/library-test.log"
    rg --quiet '^startup-native consumer-cases=457 callers=12 completed=457 failed=0$' "{{ output }}/rust/consumer-catalog.log"
    rg --quiet '^startup-native consumer-cases=457 callers=12 completed=457 failed=0$' "{{ output }}/scheme/consumer-catalog.log"
    test "$(rg -c '^native-consumer case=org_headline_function_aot::.* OK$' '{{ output }}/rust/consumer-catalog.log')" -eq 12
    test "$(rg -c '^native-consumer case=org_headline_function_aot::.* OK$' '{{ output }}/scheme/consumer-catalog.log')" -eq 12

native-semantic-wrapper-admission-receipt output:
    just native-semantic-wrapper-admission-check "{{ output }}" 2>&1 | tee "{{ output }}/wrapper-admission.log"

# Retain the exact Cargo build transcript; pipefail preserves the owner failure.
native-runtime-closure-build-receipt manifest gsc bridge_source native_output output runtime="runtime-rust":
    mkdir -p "{{ output }}"
    ORGIZE_GERBIL_NATIVE_OUTPUT="{{ native_output }}" just native-runtime-closure-build-owner "{{ manifest }}" "{{ gsc }}" "{{ bridge_source }}" "{{ runtime }}" release true 2>&1 | tee "{{ output }}/release-build.log"
    rg --quiet 'Finished .*release.*profile' "{{ output }}/release-build.log"
    test "$(rg -c '^  Executable ' "{{ output }}/release-build.log")" -eq 3
    ! rg --quiet '^error:|^error\[' "{{ output }}/release-build.log"

native-source-revision-format:
    rustfmt --edition 2024 --config skip_children=true build-support/src/source_revision.rs build-support/tests/unit/source_revision.rs

native-source-revision-test bridge_source output:
    mkdir -p "{{ output }}"
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" cargo test --offline -p orgize-build-support --lib source_revision::tests --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"' -- --nocapture 2>&1 | tee "{{ output }}/source-revision-test.log"
    rg --quiet '^test source_revision::tests::absent_legacy_input_tracks_existing_ancestor_and_then_restoration \.\.\. ok$' "{{ output }}/source-revision-test.log"
    rg --quiet '^test source_revision::tests::untracked_source_is_dirty_but_ignored_artifacts_are_not \.\.\. ok$' "{{ output }}/source-revision-test.log"
    rg --quiet '^test result: ok\. 2 passed; 0 failed;' "{{ output }}/source-revision-test.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/source-revision-test.log"

native-runtime-format:
    rustfmt --edition 2024 --config skip_children=true src/org_native_expression.rs tests/unit/org_native_expression.rs
    rustfmt --edition 2024 --config skip_children=true src/lint/mod.rs src/lint/model.rs src/lint/pipeline.rs src/lint/runtime_validation.rs src/lint/rules/document.rs src/lint/rules/attachments.rs src/lint/rules/babel.rs src/lint/rules/contracts.rs src/lint/rules/contracts_builtin.rs src/lint/rules/crypt.rs src/lint/rules/file_links.rs src/lint/rules/lifecycle.rs src/lint/rules/priority.rs src/lint/rules/progress.rs src/lint/rules/properties.rs src/lint/rules/sdd.rs src/lint/rules/table_formulas.rs src/lint/rules/task_blockers.rs src/cli/driver_sdd.rs
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/org_elements_query_expr/core_types.rs src/semantic_ast/org_elements_query_expr/core_predicate.rs src/semantic_ast/org_elements_query_expr/index.rs
    rustfmt --edition 2024 --config skip_children=true src/semantic_ast/org_elements_query_expr/core.rs src/semantic_ast/elements_bridge_selector.rs
    rustfmt --edition 2024 --config skip_children=true src/lint/rules/syntax.rs src/semantic_ast/org_elements_query_expr/core_parser.rs src/semantic_ast/org_elements_query_expr/core_contract.rs src/semantic_ast/org_elements_query_expr/mod.rs src/semantic_ast/org_contract.rs src/lint/mod.rs src/lint/render.rs tests/unit/org_contract_native_cst.rs tests/unit/lint_syntax.rs tests/unit/org_elements_query_expr.rs
    rustfmt --edition 2024 --config skip_children=true tests/integration/native_consumer_fixture.rs tests/integration/semantic_ast/native_cases.rs tests/integration/contract_evaluation_cases.rs tests/integration/contract_workspace_cases.rs tests/integration/org_parser_aot_cases.rs
    rustfmt --edition 2024 --config skip_children=true tests/integration_test.rs tests/integration/semantic_ast/mod.rs tests/integration/export_cli.rs tests/integration/semantic_ast/semantic_ast_projects_elements_bridge.rs tests/integration/library_cli.rs tests/integration/agent_cli.rs tests/integration/capture_cli.rs tests/integration/contract_composition.rs tests/integration/contract_evaluation.rs tests/integration/contract_registry.rs tests/integration/contract_workspace.rs tests/integration/contract_workspace_receipt.rs tests/integration/contract_workspace_reciprocal.rs tests/integration/document_git_scope.rs tests/integration/eval_cli.rs tests/integration/fmt_cli.rs tests/integration/fmt_links.rs tests/integration/fmt_table.rs tests/integration/harness_report_consumer.rs tests/integration/lint_attachments.rs tests/integration/lint_babel.rs tests/integration/lint_builtin_contracts.rs tests/integration/lint_contract.rs tests/integration/lint_crypt.rs tests/integration/lint_file_links.rs tests/integration/lint_fix_cli.rs tests/integration/lint_fmt.rs tests/integration/lint_lifecycle.rs tests/integration/lint_progress.rs tests/integration/lint_property_schema.rs tests/integration/lint_table_formulas.rs tests/integration/lint_task_blockers.rs tests/integration/named_source_block_template.rs tests/integration/org_aot_edit.rs tests/integration/org_case_insensitive_aot.rs tests/integration/org_citation_aot.rs tests/integration/org_customer_contract.rs tests/integration/org_dynamic_block.rs tests/integration/org_element_query.rs tests/integration/org_event_aot_contract.rs tests/integration/org_event_aot_parity.rs tests/integration/org_export_snippet_aot.rs tests/integration/org_footnote_aot.rs tests/integration/org_headline_aot.rs tests/integration/org_headline_function_aot.rs tests/integration/org_inline_code_aot.rs tests/integration/org_inline_object_aot.rs tests/integration/org_inlinetask_aot.rs tests/integration/org_list_aot.rs tests/integration/org_list_fields.rs tests/integration/org_named_drawer.rs tests/integration/org_named_elements.rs tests/integration/org_parser_aot.rs tests/integration/org_public_aot_boundary.rs tests/integration/org_script_aot.rs tests/integration/org_timestamp_aot.rs tests/integration/parse.rs tests/integration/scenario_benchmark.rs tests/integration/sdd.rs tests/integration/source_block_document.rs tests/integration/task_cli.rs tests/integration/semantic_ast/annotations_map_and_fold_work_across_the_tree.rs tests/integration/semantic_ast/existing_html_traversal_still_uses_the_lossless_substrate.rs tests/integration/semantic_ast/html_export_preserves_citation_raw_text.rs tests/integration/semantic_ast/semantic_annotations_handle_parser_line_endings_and_utf8_columns.rs tests/integration/semantic_ast/semantic_ast_covers_current_lossless_projection_surface.rs tests/integration/semantic_ast/semantic_ast_keeps_affiliated_keywords_out_of_paragraph_objects.rs tests/integration/semantic_ast/semantic_ast_keeps_quote_punctuation_plain.rs tests/integration/semantic_ast/semantic_ast_projection_and_bare_snapshot.rs tests/integration/semantic_ast/semantic_ast_projects_agenda.rs tests/integration/semantic_ast/semantic_ast_projects_agenda_view_plan.rs tests/integration/semantic_ast/semantic_ast_projects_agent_memory.rs tests/integration/semantic_ast/semantic_ast_projects_agent_planning.rs tests/integration/semantic_ast/semantic_ast_projects_attachments.rs tests/integration/semantic_ast/semantic_ast_projects_babel_eval.rs tests/integration/semantic_ast/semantic_ast_projects_block_code_refs.rs tests/integration/semantic_ast/semantic_ast_projects_block_header_args.rs tests/integration/semantic_ast/semantic_ast_projects_block_line_numbering.rs tests/integration/semantic_ast/semantic_ast_projects_block_lines.rs tests/integration/semantic_ast/semantic_ast_projects_citations.rs tests/integration/semantic_ast/semantic_ast_projects_clean_clock_duration.rs tests/integration/semantic_ast/semantic_ast_projects_clock_issues.rs tests/integration/semantic_ast/semantic_ast_projects_clock_rollups.rs tests/integration/semantic_ast/semantic_ast_projects_cloze_objects_with_metadata.rs tests/integration/semantic_ast/semantic_ast_projects_column_views.rs tests/integration/semantic_ast/semantic_ast_projects_crypt.rs tests/integration/semantic_ast/semantic_ast_projects_dynamic_blocks.rs tests/integration/semantic_ast/semantic_ast_projects_file_todo_keywords.rs tests/integration/semantic_ast/semantic_ast_projects_fixed_width_lines.rs tests/integration/semantic_ast/semantic_ast_projects_footnote_definition_label_and_body.rs tests/integration/semantic_ast/semantic_ast_projects_habits.rs tests/integration/semantic_ast/semantic_ast_projects_include_dated_agenda_extras.rs tests/integration/semantic_ast/semantic_ast_projects_inline_babel_and_footnote_details.rs tests/integration/semantic_ast/semantic_ast_projects_inlinetasks.rs tests/integration/semantic_ast/semantic_ast_projects_lesser_elements_and_blocks.rs tests/integration/semantic_ast/semantic_ast_projects_lifecycle_archive.rs tests/integration/semantic_ast/semantic_ast_projects_link_metadata.rs tests/integration/semantic_ast/semantic_ast_projects_link_protocols.rs tests/integration/semantic_ast/semantic_ast_projects_m15_alignment.rs tests/integration/semantic_ast/semantic_ast_projects_m25_alignment.rs tests/integration/semantic_ast/semantic_ast_projects_object_gap_repairs.rs tests/integration/semantic_ast/semantic_ast_projects_org_interactive.rs tests/integration/semantic_ast/semantic_ast_projects_org_table_formulas.rs tests/integration/semantic_ast/semantic_ast_projects_preprocessing_directives.rs tests/integration/semantic_ast/semantic_ast_projects_priority_properties.rs tests/integration/semantic_ast/semantic_ast_projects_progress_stats.rs tests/integration/semantic_ast/semantic_ast_projects_publishing.rs tests/integration/semantic_ast/semantic_ast_projects_radio_links.rs tests/integration/semantic_ast/semantic_ast_projects_refile.rs tests/integration/semantic_ast/semantic_ast_projects_runtime_metadata.rs tests/integration/semantic_ast/semantic_ast_projects_section_index_records.rs tests/integration/semantic_ast/semantic_ast_projects_source_blocks.rs tests/integration/semantic_ast/semantic_ast_projects_sparse_tree.rs tests/integration/semantic_ast/semantic_ast_projects_table_column_metadata.rs tests/integration/semantic_ast/semantic_ast_projects_table_el.rs tests/integration/semantic_ast/semantic_ast_projects_table_visualization.rs tests/integration/semantic_ast/semantic_ast_projects_tag_vocabulary.rs tests/integration/semantic_ast/semantic_ast_projects_tangle_and_table_records.rs tests/integration/semantic_ast/semantic_ast_projects_task_blockers.rs tests/integration/semantic_ast/semantic_ast_projects_timestamp_metadata.rs tests/integration/semantic_ast/semantic_ast_projects_workspace_index.rs tests/integration/semantic_ast/semantic_ast_resolves_internal_links.rs tests/integration/semantic_ast/semantic_citation_affixes_respect_parse_config.rs tests/integration/semantic_ast/semantic_traversal_covers_parser_v2_surface.rs tests/integration/semantic_ast/semantic_traversal_supports_exporter_and_indexer_shapes.rs tests/integration/semantic_ast/traversal_visits_annotation_bearing_metadata_nodes.rs tests/integration/export_cli_markdown.rs tests/integration/export_cli_org.rs tests/integration/export_cli_stdin.rs tests/integration/contract_workspace_reference_sentinel.rs tests/integration/contract_workspace_required_target.rs tests/integration/semantic_ast/semantic_ast_projects_elements_bridge_fixtures.rs tests/integration/semantic_ast/semantic_ast_projects_elements_bridge_indexing.rs tests/integration/semantic_ast/semantic_ast_projects_elements_bridge_query_cases.rs tests/integration/semantic_ast/semantic_ast_projects_elements_bridge_resolution.rs
    rustfmt --edition 2024 --config skip_children=true src/org_native_identity.rs src/contract_feature.rs src/document/mod.rs tests/unit/org_native_events.rs tests/unit/contract_feature.rs tests/unit/document_block_body.rs tests/unit/document_org_elements_aot.rs tests/unit/document_org_elements_query_project.rs
    rustfmt --edition 2024 --config skip_children=true tests/unit/lib.rs tests/unit/aot_affiliation.rs tests/unit/aot_projection.rs tests/unit/document_source_selection.rs tests/unit/lint_metadata.rs tests/unit/org_contract_evaluation.rs tests/unit/org_contract_source_validation.rs tests/integration/html.rs tests/integration/latex.rs tests/integration/markdown.rs
    rustfmt --edition 2024 --config skip_children=true src/native_startup.rs src/runtime_backend.rs src/org_native_events/mod.rs src/org_aot.rs src/c_ffi.rs src/lib.rs src/main.rs bindings/python/src/lib.rs bindings/python/src/contract_ffi.rs tests/integration/gerbil_rowan_cutover.rs tests/integration/native_host_children.rs benches/org_runtime.rs benches/support/runtime_corpus.rs examples/runtime_cold.rs examples/runtime_lifecycle.rs

# Compile the ownership policy alone: no native build or parser substitutes.
native-runtime-feature-policy:
    mkdir -p target/runtime-policy
    rustc --edition 2024 --crate-type lib --cfg 'feature="runtime-rust"' src/runtime_backend.rs -o target/runtime-policy/rust.rlib
    rustc --edition 2024 --crate-type lib --cfg 'feature="runtime-scheme"' src/runtime_backend.rs -o target/runtime-policy/scheme.rlib
    ! rustc --edition 2024 --crate-type lib --cfg 'feature="runtime-rust"' --cfg 'feature="runtime-scheme"' src/runtime_backend.rs -o target/runtime-policy/both.rlib
    ! rustc --edition 2024 --crate-type lib src/runtime_backend.rs -o target/runtime-policy/neither.rlib

scheme-event-benchmark parser_lib poo_flow_lib label="current" fixture="doc.org" output="":
    mkdir -p target/gerbil-test target/scheme-benchmark-receipts
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-event-fold.ss "{{ label }}" "{{ justfile_directory() }}/benches/fixtures/{{ fixture }}" "{{ justfile_directory() }}/t/benchmarks/org-event-fold/benchmark.ss" "{{ output }}"

scheme-name-set-benchmark parser_lib poo_flow_lib label output:
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-name-set.ss "{{ label }}" "{{ justfile_directory() }}/t/benchmarks/org-name-set/benchmark.ss" "{{ output }}"

scheme-name-set-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-name-set/compare.ss "{{ baseline }}" "{{ candidate }}"

scheme-helper-state-benchmark parser_lib poo_flow_lib label output mode="timing":
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-helper-state.ss "{{ label }}" "{{ justfile_directory() }}/t/benchmarks/org-helper-state/benchmark.ss" "{{ output }}" "{{ mode }}"

scheme-helper-runtime-counters-test:
    mkdir -p target/gerbil-test
    GAMBOPT=max-heap=1G,debug=q GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" gerbil test -v 3 t/benchmarks/org-helper-state/runtime-counters-test.ss 2>&1 | tee target/gerbil-test/helper-runtime-counters-test.log
    rg --quiet '^MODULE-OK' target/gerbil-test/helper-runtime-counters-test.log
    rg --quiet '^HARNESS-OK' target/gerbil-test/helper-runtime-counters-test.log
    rg --quiet '^OK$' target/gerbil-test/helper-runtime-counters-test.log
    ! rg --quiet 'ERROR|FAILED|FAILURE' target/gerbil-test/helper-runtime-counters-test.log

scheme-helper-state-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-helper-state/compare.ss "{{ baseline }}" "{{ candidate }}"

scheme-prefix-benchmark parser_lib poo_flow_lib label output:
    mkdir -p target/gerbil-test target/scheme-benchmark-receipts
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-prefix-probe.ss "{{ label }}" "{{ justfile_directory() }}/t/benchmarks/org-prefix-probe/benchmark.ss" "{{ output }}"

scheme-prefix-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-prefix-probe/compare.ss "{{ baseline }}" "{{ candidate }}"

scheme-event-benchmark-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-event-fold/compare.ss "{{ baseline }}" "{{ candidate }}"

# Matches semantics and reports failed gates; does not qualify performance.
scheme-event-benchmark-diagnostic-compare poo_flow_lib baseline candidate:
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ poo_flow_lib }}" gxi -:max-heap=1G,debug=q t/benchmarks/org-event-fold/compare.ss "{{ baseline }}" "{{ candidate }}" diagnostic

generate-contract-plan parser_lib poo_flow_lib source="languages/org/v1/modules/org-contract/generated/contract-source.ss" output="languages/org/v1/modules/org-contract/generated/contract-plan.rs":
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gxi languages/org/v1/modules/org-contract/generate-plan.ss "{{ output }}" "{{ source }}"

# Standalone C consumers only; Python uses the shared maturin extension.
contract-library:
    {{ native_env }} python3 bindings/c/build-native-library.py --output target/liborgize.{{ lib_ext }}

contract-smoke: contract-library
    {{ native_env }} cc -I bindings/c/include bindings/c/tests/orgize-dynamic-harness.c -o target/orgize-dynamic-harness
    target/orgize-dynamic-harness target/liborgize.{{ lib_ext }}
    {{ native_env }} rustc --edition=2024 bindings/rust/native_smoke.rs -L native=target {{ rust_linker }} -o target/orgize-rust-smoke
    LD_LIBRARY_PATH=target target/orgize-rust-smoke

python-test:
    {{ native_env }} uv sync --directory bindings/python --locked --extra test
    {{ native_env }} uv run --directory bindings/python --offline pytest

python-wheel:
    {{ native_env }} uv build --directory bindings/python --wheel

# Explicit local native-build owner qualification without another Python backend.
python-env:
    uv sync --directory bindings/python --locked --offline --extra test --no-install-project

python-lock offline="true":
    uv lock --directory bindings/python {{ if offline == "true" { "--offline" } else { "" } }}

# Local native build tool only; never substitute a different Python backend.
python-native-build-tool version="1.15.0":
    uv pip install --offline --python bindings/python/.venv/bin/python 'maturin=={{ version }}'

native-python-develop-owner manifest gsc bridge_source maturin="maturin" runtime="runtime-rust" verbosity="-vv":
    cd bindings/python && {{ native_env }} VIRTUAL_ENV="$PWD/.venv" XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" "{{ maturin }}" develop {{ verbosity }} --offline --uv --no-default-features --features "{{ runtime }}" --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-native-build.path="{{ bridge_source }}/build-support/gerbil-scheme-native-build"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness"' --config 'patch."https://github.com/tao3k/gerbil-scheme-rust".gerbil-scheme-rust-project-harness-policy.path="{{ bridge_source }}/build-support/gerbil-scheme-rust-project-harness-policy"'

# Preserve the standard maturin build transcript for the same native program.
native-python-develop-receipt manifest gsc bridge_source native_output output runtime="runtime-rust":
    mkdir -p "{{ output }}"
    ORGIZE_GERBIL_NATIVE_OUTPUT="{{ native_output }}" just native-python-develop-owner "{{ manifest }}" "{{ gsc }}" "{{ bridge_source }}" "{{ justfile_directory() }}/bindings/python/.venv/bin/maturin" "{{ runtime }}" -vv 2>&1 | tee "{{ output }}/develop-build.log"
    rg --quiet 'Installed orgizepy' "{{ output }}/develop-build.log"
    ! rg --quiet '^error:|^error\[' "{{ output }}/develop-build.log"

python-test-installed filter="":
    bindings/python/.venv/bin/python -m pytest bindings/python/tests -vv --durations=5 -o faulthandler_timeout=5 -k "{{ filter }}"

python-native-test-installed runtime program:
    ORGIZE_EXPECTED_NATIVE_RUNTIME="{{ runtime }}" ORGIZE_EXPECTED_NATIVE_PROGRAM="{{ program }}" just python-test-installed

python-native-test-receipt runtime program output:
    mkdir -p "{{ output }}"
    just python-native-test-installed "{{ runtime }}" "{{ program }}" 2>&1 | tee "{{ output }}/installed-test.log"
    rg --quiet '=+ [0-9]+ passed.*=+$' "{{ output }}/installed-test.log"
    ! rg --quiet '^FAILED|^ERROR|[0-9]+ failed|[0-9]+ errors?' "{{ output }}/installed-test.log"
    ! rg --quiet 'Timeout \(0:00:05\)!' "{{ output }}/installed-test.log"

python-native-format:
    rustfmt --edition 2024 --config skip_children=true bindings/python/src/lib.rs

python-wheel-repair: python-wheel
    uv sync --directory bindings/python --locked --group wheel-repair
    mkdir -p bindings/python/dist/repaired
    {{ native_env }} uv run --directory bindings/python --no-sync {{ repair_command }} "{{ justfile_directory() }}/bindings/python/dist/repaired" "{{ justfile_directory() }}/bindings/python/dist/"orgizepy-*.whl

wasm-build:
    git submodule update --init --recursive wasm
    cd wasm && CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=../target/orgize-wasm wasm-pack build -t web -d dist --out-name orgize
    rm -f wasm/dist/.gitignore

wasm: wasm-build

wasm-clean:
    rm -rf wasm/dist
