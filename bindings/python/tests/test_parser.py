from orgizepy.functions import headline_functions
from orgizepy.parser import parse_org


def test_installed_native_runtime_identity_matches_qualified_build():
    import os
    import re
    from orgizepy._orgize import _native_runtime_identity

    backend, program = _native_runtime_identity()
    assert backend in {"runtime-rust", "runtime-scheme"}
    assert re.fullmatch(r"sha256:[0-9a-f]{64}", program)
    expected_backend = os.environ.get("ORGIZE_EXPECTED_NATIVE_RUNTIME")
    expected_program = os.environ.get("ORGIZE_EXPECTED_NATIVE_PROGRAM")
    if expected_backend:
        assert backend == expected_backend, "stale or wrong-feature native extension"
    if expected_program:
        assert program == expected_program, "stale native program in installed extension"


def test_native_parse_releases_gil_for_host_progress():
    import sys
    import threading

    ready = threading.Event()
    start = threading.Event()
    finished = threading.Event()
    observations = []

    def observe():
        ready.set()
        assert start.wait(5), "parse handoff did not start"
        observations.append(not finished.is_set())

    # Prevent ordinary bytecode switching from making a GIL-held native call
    # appear concurrent. The observer can run only while parse detaches, or
    # after finished is set and join releases the GIL (which must fail).
    original_interval = sys.getswitchinterval()
    worker = threading.Thread(target=observe)
    try:
        sys.setswitchinterval(60)
        worker.start()
        assert ready.wait(5), "host observer did not become ready"
        source = "* Native heartbeat\n" + "[[id:proof][evidence]] text\n" * 512
        start.set()
        document = parse_org(source)
        finished.set()
        assert any(element.kind == "headline" for element in document.elements)
        worker.join(5)
        assert not worker.is_alive()
        assert observations == [True], "host thread was blocked by native parse's GIL"
    finally:
        finished.set()
        start.set()
        sys.setswitchinterval(original_interval)
        worker.join(5)


def test_raw_org_parse_and_functions():
    document = parse_org("#+TODO: NEXT DONE\n* NEXT Ship SDK\n")
    headlines = [element for element in document.elements if element.kind == "headline"]
    assert len(headlines) == 1
    assert ("title", "NEXT Ship SDK") in headlines[0].fields
    functions = headline_functions(document, headlines[0].id)
    assert functions.todo_keyword == "NEXT"
    assert functions.content_after_todo == "Ship SDK"


def test_empty_and_unicode_org():
    assert parse_org("").elements[0].kind == "org-data"
    document = parse_org("* 设计决定\n")
    assert any(("title", "设计决定") in element.fields for element in document.elements)
