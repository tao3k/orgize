# checks: native architecture task owner.

# Dependency closure only: this does not certify native-view ownership or
# removal of the event-to-navigation-index materialization path.
[private]
native-dependency-check:
    test ! -d languages/org/v1
    ! rg --quiet 'languages/org/v1|org-v1-' src languages t tests bindings build.ss build-parser.ss build-support benches examples --glob '!*.snap'
    ! rg --quiet 'gerbil-parser-rowan|^rowan\s*=|name = "rowan"' Cargo.toml Cargo.lock fuzz/Cargo.lock
    ! rg --quiet 'gerbil_parser_rowan|\browan::|pub use rowan' src tests benches examples languages --glob '*.rs'
    ! rg --fixed-strings --quiet 'rowan.build' src tests benches just/benchmark.just
    ! rg --quiet 'rowan-event|org-rowan|parse-org-rowan-events|rowan_admission' src languages bindings t tests build-parser.ss build.ss
    ! rg --quiet '\b[Rr]owan\b' src languages bindings t tests --glob '!*.snap'
    rg --fixed-strings --quiet 'rev = "{{ parser_revision }}"' Cargo.toml
    rg --fixed-strings --quiet 'github.com/tao3k/gerbil-parser@{{ parser_revision }}' gerbil.pkg
    @echo 'ROWAN-FREE-DEPENDENCIES-OK: native view cutover is a separate gate'

# Bounded native migration records and implementation owner checklists.
[private]
native-document-layout-check:
    @awk 'FNR == 1 { if (NR > 1 && count > 500) { print "FAIL " previous " lines=" count; failed = 1 }; previous = FILENAME; count = 0 } { count++ } END { if (count > 500) { print "FAIL " previous " lines=" count; failed = 1 }; if (!failed) print "DOC-LAYOUT-OK: owner records <= 500 lines"; exit failed }' docs/20_parser/20.08_native_scheme_closure_ledger.org docs/20_parser/20.08_native_scheme_closure/*.org docs/90_operations/90.01_implementation_status.org docs/90_operations/90.01_implementation_status/*.org

# Validate platform-specific command expansion; this is not a Linux execution.
[group('checks')]
native-platform-command-check output:
    mkdir -p "{{ output }}"
    just --set host_os macos --dry-run scheme-parser-build-isolated /tmp/orgize-platform-check /tmp/orgize-loadpath 2>&1 | tee "{{ output }}/macos-scheme.log"
    just --set host_os linux --dry-run scheme-parser-build-isolated /tmp/orgize-platform-check /tmp/orgize-loadpath 2>&1 | tee "{{ output }}/linux-scheme.log"
    just --set host_os linux --dry-run scheme-native-clock-test /tmp/orgize-platform-check 2>&1 | tee "{{ output }}/linux-clock.log"
    just --set host_os linux --dry-run scheme-native-runtime-owner-build /tmp/orgize-platform-check /tmp/orgize-loadpath 2>&1 | tee "{{ output }}/linux-service.log"
    just --set host_os linux --dry-run native-runtime-queue-test-receipt /tmp/gerbil/bin/gsc /tmp/orgize-platform-check 2>&1 | tee "{{ output }}/linux-queue.log"
    just --set host_os linux --dry-run native-runtime-closure-build-owner /tmp/program.json /tmp/gsc runtime-rust release false 2>&1 | tee "{{ output }}/linux-rust.log"
    just --set host_os macos --dry-run native-python-develop-owner /tmp/program.json /tmp/gsc maturin runtime-rust 2>&1 | tee "{{ output }}/macos-python.log"
    just --set host_os linux --dry-run native-python-develop-owner /tmp/program.json /tmp/gsc maturin runtime-rust 2>&1 | tee "{{ output }}/linux-python.log"
    rg --quiet 'env -u SDKROOT CC=/usr/bin/cc' "{{ output }}/macos-scheme.log"
    rg --fixed-strings --quiet -- '-- -C linker=/usr/bin/cc -C link-arg=-Wl,-ld_classic' "{{ output }}/macos-python.log"
    ! rg --quiet 'SDKROOT|APPLE_DARWIN|/usr/bin/cc|-Wl,' "{{ output }}/linux-scheme.log" "{{ output }}/linux-clock.log" "{{ output }}/linux-service.log" "{{ output }}/linux-queue.log" "{{ output }}/linux-rust.log" "{{ output }}/linux-python.log"

# Development-only: both arguments are compiled Gerbil lib directories.
[private]
native-parser-revision-check source revision:
    test "$(git -C '{{ source }}' rev-parse HEAD)" = "{{ revision }}"
    rg --fixed-strings --quiet 'rev = "{{ revision }}"' Cargo.toml
    rg --fixed-strings --quiet 'git+https://github.com/tao3k/gerbil-parser?rev={{ revision }}#{{ revision }}' Cargo.lock
    rg --fixed-strings --quiet 'git+https://github.com/tao3k/gerbil-parser?rev={{ revision }}#{{ revision }}' fuzz/Cargo.lock
    rg --fixed-strings --quiet 'github.com/tao3k/gerbil-parser@{{ revision }}' gerbil.pkg
    git -C "{{ source }}" diff --check
    git -C "{{ source }}" diff --quiet
    git -C "{{ source }}" diff --cached --quiet
    git -C "{{ source }}" status --short

# Pin equality alone cannot admit an old compiled Scheme closure. Every
# parser/Org module must come from this candidate's explicit build roots.
[private]
native-parser-program-source-check manifest parser_output org_output program_output:
    jq --exit-status --arg parser "{{ parser_output }}/lib/static/" --arg org "{{ org_output }}/lib/static/" --arg stage "{{ program_output }}/lib/static/" '.schema == "gerbil-scheme-rust.aot-program.v1" and ([.modules[] | select(.module | startswith("gerbil-parser/"))] | length > 0) and ([.modules[] | select(.module | startswith("orgize/"))] | length > 0) and all(.modules[]; if (.module | startswith("gerbil-parser/")) then (.scm | startswith($parser)) elif (.module == "orgize/bindings/c/native-clock" or .module == "orgize/bindings/c/native-runtime-statistics" or .module == "orgize/languages/org/native-event-tape") then (.scm | startswith($stage)) elif (.module | startswith("orgize/")) then (.scm | startswith($org)) else true end)' "{{ manifest }}"

[private]
native-parser-program-source-negative-check previous parser_output org_output program_output:
    jq --exit-status '.schema == "gerbil-scheme-rust.aot-program.v1" and (.modules | length > 0)' "{{ previous }}"
    ! just native-parser-program-source-check "{{ previous }}" "{{ parser_output }}" "{{ org_output }}" "{{ program_output }}"

[group('checks')]
native-alignment-dependency-check parser_source bridge_source:
    just native-task-layout-check
    just native-parser-revision-check "{{ parser_source }}" "{{ parser_revision }}"
    test "$(git -C '{{ bridge_source }}' rev-parse HEAD)" = 1e4f1f65c7ff8e95fe764f2e8549fa9d92275db4
    rg --fixed-strings --quiet 'rev = "1e4f1f65c7ff8e95fe764f2e8549fa9d92275db4"' build-support/Cargo.toml
    rg --fixed-strings --quiet 'github.com/tao3k/gerbil-scheme-rust@1e4f1f65c7ff8e95fe764f2e8549fa9d92275db4' gerbil.pkg
    ! rg --quiet 'source = "git\+https://github.com/tao3k/gerbil-scheme-rust\?rev=(?!1e4f1f65)' Cargo.lock --pcre2
    ! rg --quiet -- "--config ['\"]patch[.]" Justfile just
    just native-dependency-check

# Check both recipe dependencies and delegated owner calls after task removal.
[private]
native-task-layout-check:
    just --dump --dump-format json | jq --exit-status '.recipes as $recipes | all($recipes[]; all(.dependencies[]; .recipe as $name | $recipes | has($name)) and all((.body[] | flatten | map(select(type == "string")) | join(" ") | scan("\\bjust ([a-z][a-z0-9-]*)") | .[0]); . as $name | $recipes | has($name)))'
    ! rg --quiet '^native-(fold-state|rowan)-|^scheme-(parser-rebase|event-fold)-' just

[private]
native-owner-lifecycle-check source gsc:
    mkdir -p target/runtime-policy
    {{ native_env }} cc -O2 -DGERBIL_SCHEME_RUST_EXTERNAL_PROGRAM -I"{{ parent_directory(parent_directory(gsc)) }}/include" -c "{{ source }}/native/runtime.c" -o target/runtime-policy/lifecycle.o

[private]
native-runtime-inner-stage-scale-check receipt: (native-runtime-profile-scale-check receipt)
    jq --exit-status 'all(.[]; .stage_timings_ns | has("native.scheme_fold") and has("native.tape_encode") and has("native.scheme_thread_cpu"))' "{{ receipt }}"

# Reject the audited production-library warning group, not dependency/linker warnings.
[private]
native-semantic-wrapper-warning-check output:
    rg --quiet 'Finished .*release.*profile' "{{ output }}/release-build.log"
    ! rg --quiet '^warning: `orgize` \(lib\) generated' "{{ output }}/release-build.log"
    ! rg --quiet '^[[:space:]]*--> (src/org_native_semantic_functions.rs|tests/integration/(org_headline_function_aot.rs|support/native_semantic_values.rs)):' "{{ output }}/release-build.log"
    ! rg --quiet '^error:|^error\[' "{{ output }}/release-build.log"

[private]
native-build-support-test output:
    mkdir -p "{{ output }}"
    {{ native_env }} XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" cargo test --offline --locked --release -p orgize-build-support --lib -- --nocapture 2>&1 | tee "{{ output }}/build-support-test.log"
    rg --quiet '^test result: ok\. 5 passed; 0 failed;' "{{ output }}/build-support-test.log"
    ! rg --quiet 'FAILED|panicked at' "{{ output }}/build-support-test.log"

# Compile the ownership policy alone: no native build or parser substitutes.
[group('checks')]
native-runtime-feature-policy:
    mkdir -p target/runtime-policy
    rustc --edition 2024 --crate-type lib --cfg 'feature="runtime-rust"' src/runtime_backend.rs -o target/runtime-policy/rust.rlib
    rustc --edition 2024 --crate-type lib --cfg 'feature="runtime-scheme"' src/runtime_backend.rs -o target/runtime-policy/scheme.rlib
    ! rustc --edition 2024 --crate-type lib --cfg 'feature="runtime-rust"' --cfg 'feature="runtime-scheme"' src/runtime_backend.rs -o target/runtime-policy/both.rlib
    ! rustc --edition 2024 --crate-type lib src/runtime_backend.rs -o target/runtime-policy/neither.rlib
