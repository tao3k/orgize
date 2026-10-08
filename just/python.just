# python: native architecture task owner.

[group('python')]
python-test:
    {{ native_env }} {{ native_rust_flags }} uv sync --directory bindings/python --locked --extra test
    {{ native_env }} {{ native_rust_flags }} uv run --directory bindings/python --offline pytest

[private]
python-wheel:
    {{ native_env }} {{ native_rust_flags }} uv build --directory bindings/python --wheel

# Explicit local native-build owner qualification without another Python backend.
[group('python')]
python-env offline="true":
    uv sync --directory bindings/python --locked {{ if offline == "true" { "--offline" } else { "" } }} --extra test --no-install-project

[group('python')]
python-lock offline="true":
    uv lock --directory bindings/python {{ if offline == "true" { "--offline" } else { "" } }}

# Local native build tool only; never substitute a different Python backend.
[private]
python-native-build-tool version="1.15.0" offline="true":
    uv pip install {{ if offline == "true" { "--offline" } else { "" } }} --python bindings/python/.venv/bin/python 'maturin=={{ version }}'

[private]
native-python-develop-owner manifest gsc maturin="maturin" runtime="runtime-rust" verbosity="-vv":
    cd bindings/python && {{ native_env }} VIRTUAL_ENV="$PWD/.venv" XDG_CACHE_HOME="{{ justfile_directory() }}/target/native-cache" ORGIZE_GERBIL_PROGRAM_MANIFEST="{{ manifest }}" GERBIL_GSC="{{ gsc }}" "{{ maturin }}" develop {{ verbosity }} --offline --uv --no-default-features --features "{{ runtime }}" -- {{ native_link_args }}

# Preserve the standard maturin build transcript for the same native program.
[group('python')]
native-python-develop-receipt manifest gsc native_output output runtime="runtime-rust":
    mkdir -p "{{ output }}"
    ORGIZE_GERBIL_NATIVE_OUTPUT="{{ native_output }}" just native-python-develop-owner "{{ manifest }}" "{{ gsc }}" "{{ justfile_directory() }}/bindings/python/.venv/bin/maturin" "{{ runtime }}" -vv 2>&1 | tee "{{ output }}/develop-build.log"
    rg --quiet 'Installed orgizepy' "{{ output }}/develop-build.log"
    ! rg --quiet '^error:|^error\[' "{{ output }}/develop-build.log"

[private]
python-test-installed filter="":
    bindings/python/.venv/bin/python -m pytest bindings/python/tests -vv --durations=5 -o faulthandler_timeout=5 -k "{{ filter }}"

# Inspect the installed extension without initializing its native runtime.
[private]
python-native-identity:
    bindings/python/.venv/bin/python -c 'from orgizepy._orgize import _native_runtime_identity; print(*_native_runtime_identity())'

[private]
python-native-test-installed runtime program:
    ORGIZE_EXPECTED_NATIVE_RUNTIME="{{ runtime }}" ORGIZE_EXPECTED_NATIVE_PROGRAM="{{ program }}" just python-test-installed

[group('python')]
python-native-test-receipt runtime program output:
    mkdir -p "{{ output }}"
    just python-native-test-installed "{{ runtime }}" "{{ program }}" 2>&1 | tee "{{ output }}/installed-test.log"
    rg --quiet '=+ [0-9]+ passed.*=+$' "{{ output }}/installed-test.log"
    ! rg --quiet '^FAILED|^ERROR|[0-9]+ failed|[0-9]+ errors?' "{{ output }}/installed-test.log"
    ! rg --quiet 'Timeout \(0:00:05\)!' "{{ output }}/installed-test.log"

[group('python')]
python-wheel-repair: python-wheel
    uv sync --directory bindings/python --locked --group wheel-repair
    mkdir -p bindings/python/dist/repaired
    {{ native_env }} uv run --directory bindings/python --no-sync {{ repair_command }} "{{ justfile_directory() }}/bindings/python/dist/repaired" "{{ justfile_directory() }}/bindings/python/dist/"orgizepy-*.whl
