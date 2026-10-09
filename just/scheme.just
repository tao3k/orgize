# scheme: native architecture task owner.

# Qualify the consumer against an explicitly selected parser facade.
[private]
scheme-event-priority-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/modules/org-parser/event-inline.ss t/org-native-inline-parser-test.ss
    just scheme-native-module-test t/org-native-inline-parser-test.ss "{{ output }}" "{{ load_path }}"

# OS clock calibration is a native test, not a production parser dependency.
[private]
scheme-native-runtime-statistics-test output:
    test "{{ host_os }}" != "windows"
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-runtime-statistics.ss t/native-clock/reference.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib" gerbil test -v 3 t/native-runtime-statistics-test.ss 2>&1 | tee "{{ output }}/native-runtime-statistics-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/native-runtime-statistics-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/native-runtime-statistics-test.log"
    rg --quiet '^OK$' "{{ output }}/native-runtime-statistics-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/native-runtime-statistics-test.log"

[private]
scheme-native-clock-test output:
    test "{{ host_os }}" != "windows"
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss t/native-clock/reference.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib" gerbil test -v 3 t/native-clock-test.ss 2>&1 | tee "{{ output }}/native-clock-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/native-clock-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/native-clock-test.log"
    rg --quiet '^OK$' "{{ output }}/native-clock-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/native-clock-test.log"

[private]
scheme-aligned-parser-build source output load_path:
    mkdir -p "{{ output }}"
    cd "{{ source }}" && {{ scheme_env }} GERBIL_BUILD_CORES=12 GERBIL_BUILD_VERBOSE=1 GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build.ss compile --optimized

[group('scheme')]
scheme-aligned-parser-build-receipt source output load_path:
    mkdir -p "{{ output }}"
    just scheme-aligned-parser-build "{{ source }}" "{{ output }}" "{{ load_path }}" 2>&1 | tee "{{ output }}/parser-build.log"
    just scheme-aligned-parser-sdk-check "{{ output }}" "{{ load_path }}" 2>&1 | tee "{{ output }}/sdk-load.log"

# Import phase code and the public runtime surface; a cached build exit alone
# cannot admit an SDK interrupted before its loadable objects were produced.
[private]
scheme-aligned-parser-sdk-check output load_path:
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive -e '(import :gerbil-parser/src/language/descriptor :gerbil-parser/src/modules/parser/interface)'

[group('scheme')]
scheme-aligned-program-stage-receipt output load_path parser_output org_output:
    mkdir -p "{{ output }}"
    just scheme-parser-batch-stage-isolated "{{ output }}" "{{ load_path }}" 2>&1 | tee "{{ output }}/program-stage.log"
    just native-parser-program-source-check "{{ output }}/program.json" "{{ parser_output }}" "{{ org_output }}" "{{ output }}"

[group('scheme')]
scheme-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test t/*-test.ss
    just qualify-source-headlines "{{ justfile_directory() }}/target/gerbil-test" "{{ parser_lib }}:{{ poo_flow_lib }}"

# Source projection retains all four original cases under native gxtest.
[private]
qualify-source-headlines output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/modules/org-elements/source-headlines.ss languages/org/modules/org-elements/source-interface.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-source-headlines-qualification.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" python3 tools/ci/watch-real-output.py gerbil test -v 5 t/org-source-headlines-qualification.ss 2>&1 | tee "{{ output }}/module-test.log"
    test "$(rg -c '^CASE-OK ' '{{ output }}/module-test.log')" -eq 4
    rg --quiet '^OK$' "{{ output }}/module-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/module-test.log"

# This owner check does not replace whole-parser startup, performance or scale gates.
# Native gxtest remains the test owner; always compile the requested module.
[private]
scheme-native-module-fresh-test suite output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O "{{ suite }}"
    just scheme-native-module-test "{{ suite }}" "{{ output }}" "{{ load_path }}"

[private]
scheme-native-module-test suite output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" python3 tools/ci/watch-real-output.py gerbil test -v 5 "{{ suite }}" 2>&1 | tee "{{ output }}/module-test.log"
    rg --quiet '^CASE-OK ' "{{ output }}/module-test.log"
    rg --quiet '^OK$' "{{ output }}/module-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/module-test.log"

# Focus the existing Scheme-owned Org event suites with a pre-import heap cap.
[private]
scheme-event-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test -v 3 t/org-native-event-parser-test.ss 2>&1 | tee target/gerbil-test/org-event-parser.log
    rg --quiet '^MODULE-OK' target/gerbil-test/org-event-parser.log
    rg --quiet '^HARNESS-OK' target/gerbil-test/org-event-parser.log
    rg --quiet '^OK$' target/gerbil-test/org-event-parser.log
    ! rg --quiet 'ERROR|FAILED|FAILURE' target/gerbil-test/org-event-parser.log

# Link the same standard test suite values ahead of time, not at test startup.
[group('scheme')]
scheme-native-closure-receipt output load_path:
    just scheme-native-closure-build "{{ output }}" "{{ load_path }}"
    just scheme-native-closure-test "{{ output }}"
    just scheme-native-closure-failure-control "{{ output }}"

[private]
scheme-native-closure-build output load_path:
    mkdir -p "{{ output }}"
    just scheme-native-closure-owner-build "{{ output }}" "{{ load_path }}"
    just scheme-native-closure-support-build "{{ output }}" "{{ load_path }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-native-publishing-value-test.ss t/org-native-citation-export-test.ss t/org-native-metadata-value-test.ss t/org-native-source-value-test.ss t/org-native-lifecycle-value-test.ss t/org-native-semantic-owner-test.ss t/org-elements-module-test.ss t/org-radio-match-test.ss t/org-native-inline-parser-test.ss t/org-native-structural-parser-test.ss t/org-native-table-container-parser-test.ss t/org-native-event-parser-test.ss t/org-source-headlines-qualification.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-native-closure.ss
    just scheme-native-closure-link "{{ output }}" "{{ load_path }}"

[private]
scheme-native-closure-link output load_path:
    just scheme-native-executable-link t/org-native-closure.ss "{{ output }}" "{{ load_path }}" org-native-closure '-I{{ justfile_directory() }}/bindings/c/include'

# Recompile the Scheme service owner without rebuilding grammar/test owners.
[private]
scheme-native-runtime-owner-build output load_path:
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -V -O -cc-options '-I{{ justfile_directory() }}/bindings/c/include' -ld-options '{{ clock_linker }}' bindings/c/orgize-scheme-runtime.ss 2>&1 | tee "{{ output }}/runtime-owner-build.log" | rg '^compile |^invoke |^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

# Compiler outputs belong to the selected executable, never dependency SDKs.
[private]
scheme-native-executable-link source output load_path binary cc_options="":
    mkdir -p "{{ output }}/sdk/lib/static"
    # Separate compilation writes beside static inputs. Copy in reverse load
    # order so the first dependency root wins, while external SDKs stay read-only.
    IFS=: read -r -a libs <<< "{{ load_path }}"; for ((i=${#libs[@]}-1; i>=0; i--)); do if test -d "${libs[i]}/static"; then cp -p "${libs[i]}"/static/*.scm "{{ output }}/sdk/lib/static/"; fi; done

    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ output }}/sdk/lib:{{ load_path }}" gerbil compile -V -O -exe -cc-options '{{ cc_options }}' -o "{{ output }}/{{ binary }}" "{{ source }}" 2>&1 | tee "{{ output }}/native-link.log" | rg '^compile |^invoke |^\.\.\. generate parser artifact|^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

[private]
scheme-native-document-value-build output load_path:
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O -S languages/org/modules/org-parser/value-funs.ss

[private]
scheme-native-closure-support-build output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -V -O languages/org/generated/native-event-fixture.ss languages/org/native-event-fixture.ss t/org-parser-test-support.ss 2>&1 | tee "{{ output }}/support-build.log" | rg '^compile |^invoke |^\.\.\. generate parser artifact|^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

[private]
scheme-native-closure-owner-build output load_path:
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -V -O languages/org/modules/org-elements/catalog.ss languages/org/graph.ss languages/org/modules/org-elements/config.ss languages/org/modules/org-elements/aot.ss languages/org/modules/org-elements/interface.ss languages/org/modules/org-elements/generated/query-source.ss t/org-elements-test-support.ss 2>&1 | tee "{{ output }}/owner-build.log" | rg '^compile |^invoke |^\.\.\. generate parser artifact|^\*\*\* ERROR|^--- Syntax Error|^\.\.\. (form|detail):'

[private]
scheme-native-closure-test output:
    env -u GERBIL_PATH -u GERBIL_LOADPATH GAMBOPT="{{ scheme_gambopt }}" python3 tools/ci/watch-real-output.py "{{ output }}/org-native-closure" 2>&1 | tee "{{ output }}/native-test.log"
    test "$(rg -c '^CASE-OK ' '{{ output }}/native-test.log')" -eq 178
    test "$(rg -c '^SUITE-OK ' '{{ output }}/native-test.log')" -eq 12
    rg --quiet '^HARNESS-OK ' "{{ output }}/native-test.log"
    rg --quiet '^OK$' "{{ output }}/native-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE|FAIL:' "{{ output }}/native-test.log"

# A failed standard Gerbil assertion must reach the executable exit status.
[private]
scheme-native-closure-failure-control output:
    status=0; env -u GERBIL_PATH -u GERBIL_LOADPATH GAMBOPT="{{ scheme_gambopt }}" python3 tools/ci/watch-real-output.py "{{ output }}/org-native-closure" --failure-control > "{{ output }}/failure-control.log" 2>&1 || status=$?; test "$status" -eq 42
    rg --quiet '^FAILED$' "{{ output }}/failure-control.log"
    rg --quiet '^ERROR CHECK ' "{{ output }}/failure-control.log"
    ! rg --quiet '^OK$' "{{ output }}/failure-control.log"

# Copy a fetched exact revision without touching an existing dirty checkout.
[private]
scheme-parser-source-clone source output revision:
    git clone --no-checkout --local --no-hardlinks "{{ source }}" "{{ output }}"
    git -C "{{ output }}" checkout --detach "{{ revision }}"

[private]
scheme-elements-module-fresh-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-elements-module-test.ss
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 5 t/org-elements-module-test.ss 2>&1 | tee "{{ output }}/elements-test.log"
    rg --quiet '^CASE-OK catalog and projected query share one feature interface$' "{{ output }}/elements-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/elements-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/elements-test.log"
    rg --quiet '^OK$' "{{ output }}/elements-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/elements-test.log"

[group('scheme')]
scheme-timestamp-graph-generate output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/generate-elements.ss languages/org/generated/elements.rs
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/grammar.ss languages/org/graph-shape.ss languages/org/graph.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/generate-parser.ss languages/org/generated/parser.rs
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/generate-graph.ss languages/org/generated/graph.rs
    just scheme-parser-dependent-plans-generate "{{ output }}" "{{ output }}/lib:{{ load_path }}"

# Query and Contract packs bind the graph identity and must move with its table.
[private]
scheme-parser-dependent-plans-generate output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/modules/org-contract/generate-parser.ss languages/org/modules/org-contract/generated/parser.rs
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/modules/org-contract/generate-expectation-parser.ss languages/org/modules/org-contract/generated/expectation.rs
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/modules/org-elements/generate-queries.ss languages/org/modules/org-elements/generated/query-pack.rs
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/modules/org-contract/generate-plan.ss languages/org/modules/org-contract/generated/contract-plan.rs
    just scheme-consumer-packs-regenerate "{{ output }}" "{{ load_path }}"

[private]
scheme-structural-event-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-native-structural-parser-test.ss
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 3 t/org-native-structural-parser-test.ss 2>&1 | tee "{{ output }}/structural-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/structural-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/structural-test.log"
    rg --quiet '^OK$' "{{ output }}/structural-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/structural-test.log"

[private]
scheme-event-fixture-generate output load_path destination:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/native-event-fixture.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive languages/org/generate-native-event-fixture.ss "{{ destination }}"

# Fresh child suites are required when editing grammar expectations. The caller
# supplies the separately qualified parser overlay; this is not an AOT link gate.
[private]
scheme-event-fresh-test output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O languages/org/generated/native-event-fixture.ss languages/org/native-event-fixture.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/org-native-inline-parser-test.ss t/org-native-structural-parser-test.ss t/org-native-table-container-parser-test.ss t/org-native-event-parser-test.ss
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 5 t/org-native-event-parser-test.ss 2>&1 | tee "{{ output }}/event-test.log"
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

[private]
scheme-event-tape-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil compile -O -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss bindings/c/native-runtime-statistics.ss
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ justfile_directory() }}/target/gerbil-test/lib:{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test -v 3 t/org-event-tape-test.ss 2>&1 | tee target/gerbil-test/org-event-tape.log
    rg --quiet '^MODULE-OK' target/gerbil-test/org-event-tape.log
    rg --quiet '^HARNESS-OK' target/gerbil-test/org-event-tape.log
    rg --quiet '^OK$' target/gerbil-test/org-event-tape.log
    ! rg --quiet 'ERROR|FAILED|FAILURE' target/gerbil-test/org-event-tape.log

# Native actor lifecycle admission, independent of foreign-thread attachment.
[private]
scheme-native-actor-test parser_lib poo_flow_lib:
    mkdir -p target/gerbil-test
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil test -v 3 t/org-native-actor-test.ss

[private]
scheme-native-actor-build parser_lib poo_flow_lib:
    mkdir -p target/gerbil-parser
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-parser" GERBIL_LOADPATH="{{ parser_lib }}:{{ poo_flow_lib }}" gerbil compile -O -S bindings/c/orgize-actor.ss

# Exercise upstream scanner/LR/recovery changes after an exact-head rebase.
[private]
scheme-parser-engine-fixtures source output load_path:
    cd "{{ source }}" && {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O t/fixtures/lr1-construction.ss t/benchmarks/contextual-scanner/parser-fixture.ss

[private]
scheme-parser-engine-test source output load_path:
    cd "{{ source }}" && GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil test -v 3 t/contextual-scanner-test.ss t/contextual-dispatch-test.ss t/contextual-parser-test.ss t/contextual-deferred-scanner-test.ss t/canonical-lr1-reference-test.ss t/recovery-incremental-test.ss 2>&1 | tee "{{ output }}/engine-test.log"
    rg --quiet '^MODULE-OK' "{{ output }}/engine-test.log"
    rg --quiet '^HARNESS-OK' "{{ output }}/engine-test.log"
    rg --quiet '^OK$' "{{ output }}/engine-test.log"
    ! rg --quiet 'ERROR|FAILED|FAILURE' "{{ output }}/engine-test.log"

# Preserve the exact compiled dependency closure for a matched algorithm A/B.
[private]
scheme-parser-lib-save parser_lib label:
    test ! -e "target/scheme-receipt-libs/{{ label }}"
    mkdir -p target/scheme-receipt-libs
    cp -R "{{ parser_lib }}" "target/scheme-receipt-libs/{{ label }}"

[private]
scheme-parser-build load_path cores="12":
    just scheme-parser-build-isolated "{{ justfile_directory() }}/target/gerbil-parser" "{{ load_path }}" "{{ cores }}"

# Requalify the complete Org graph without retaining a previous engine's IR.
[group('scheme')]
scheme-parser-build-isolated output load_path cores="12":
    mkdir -p "{{ output }}"
    {{ scheme_env }} GERBIL_BUILD_CORES="{{ cores }}" GERBIL_BUILD_VERBOSE=1 GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build.ss compile --optimized
    {{ scheme_env }} GERBIL_BUILD_CORES="{{ cores }}" GERBIL_BUILD_VERBOSE=1 GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gxi build-parser.ss compile --optimized

# Refresh the private bounded batch projector in a separate compiler-owned
# program. Its unchanged dependency closure comes from explicit compiled libs.
[private]
scheme-parser-batch-stage-isolated output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil compile -O -S -ld-options '{{ clock_linker }}' bindings/c/native-clock.ss bindings/c/native-runtime-statistics.ss languages/org/native-event-tape.ss
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive bindings/c/stage-parser.ss "{{ output }}"

# Contract's existing graph-query dependency needs static Scheme artifacts too.
[private]
scheme-contract-deps source load_path:
    cd "{{ source }}" && GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="$PWD/.gerbil" GERBIL_LOADPATH="{{ load_path }}" gerbil compile -O -S src/modules/parser/graph-query-types.ss src/modules/parser/graph-query-objects.ss src/modules/parser/graph-query-funs.ss graph-query-support.ss

# Consumer fixtures are generated by the public Scheme AOT APIs, not by
# replacing digest strings in Rust or disabling graph identity admission.
[private]
scheme-consumer-packs-regenerate output load_path:
    mkdir -p "{{ output }}"
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive tests/fixtures/org-elements/generate-customer-queries.ss tests/fixtures/org-elements/generated/customer-query-pack.rs
    {{ scheme_env }} GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ output }}" GERBIL_LOADPATH="{{ output }}/lib:{{ load_path }}" gerbil interactive tests/fixtures/org-contract/generate-customer-contracts.ss tests/fixtures/org-contract/generated/customer-contract-pack.rs

[private]
scheme-helper-runtime-counters-test:
    mkdir -p target/gerbil-test
    GAMBOPT="{{ scheme_gambopt }}" GERBIL_PATH="{{ justfile_directory() }}/target/gerbil-test" gerbil test -v 3 t/benchmarks/org-helper-state/runtime-counters-test.ss 2>&1 | tee target/gerbil-test/helper-runtime-counters-test.log
    rg --quiet '^MODULE-OK' target/gerbil-test/helper-runtime-counters-test.log
    rg --quiet '^HARNESS-OK' target/gerbil-test/helper-runtime-counters-test.log
    rg --quiet '^OK$' target/gerbil-test/helper-runtime-counters-test.log
    ! rg --quiet 'ERROR|FAILED|FAILURE' target/gerbil-test/helper-runtime-counters-test.log
