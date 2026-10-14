"""The shared native owner must preserve Python's child exit statuses."""

import os
import signal
from pathlib import Path
import subprocess
import sys
import time

import pytest


def _probe():
    import fcntl

    seen = []
    signal.signal(signal.SIGTERM, lambda number, frame: seen.append(number))
    def spawn():
        return os.posix_spawn(
            sys.executable, [sys.executable, "-c", "raise SystemExit(23)"], os.environ
        )

    pid, status = os.waitpid(spawn(), 0)
    assert pid > 0 and os.waitstatus_to_exitcode(status) == 23
    print("HOST-WAIT-BEFORE OK", flush=True)
    # Capture after the probe's first stdout write: Darwin can expose a
    # write-history flag in F_GETFL. Only import/startup belong to this gate.
    flags = [fcntl.fcntl(fd, fcntl.F_GETFL) for fd in range(3)]

    from orgizepy.parser import parse_org
    from orgizepy import initialize_native_runtime

    imported_flags = [fcntl.fcntl(fd, fcntl.F_GETFL) for fd in range(3)]
    assert imported_flags == flags, f"import changed stdio flags: {flags} -> {imported_flags}"

    with pytest.raises(ValueError, match="not initialized"):
        parse_org("* Before explicit startup\n")

    from orgizepy.contract import (
        ContractRow, OrgizeContractError, evaluate_contract,
    )
    with pytest.raises(OrgizeContractError):
        evaluate_contract(
            [ContractRow(0, -1, "org-data"), ContractRow(1, 0, "headline")],
            scope_id=0, kind="headline", expectation="exactly", expected_count=1,
        )

    # No application threads and the earlier child has been reaped.
    initialize_native_runtime()
    initialize_native_runtime()
    initialized_flags = [fcntl.fcntl(fd, fcntl.F_GETFL) for fd in range(3)]
    assert initialized_flags == flags, f"startup changed stdio flags: {flags} -> {initialized_flags}"
    os.kill(os.getpid(), signal.SIGTERM)
    assert seen == [signal.SIGTERM], "custom Python signal handler must survive startup"
    initialize_native_runtime()
    print("HOST-STARTUP-RESOURCES OK", flush=True)

    assert parse_org("* Native owner\n").elements[0].kind == "org-data"
    children = [spawn() for _ in range(12)]
    # Let exited children deliver SIGCHLD before the host explicitly waits.
    # The bounded probe below is isolated from pytest's prior native calls.
    time.sleep(0.1)
    failures = []
    for child in children:
        try:
            pid, status = os.waitpid(child, 0)
            if pid != child or os.waitstatus_to_exitcode(status) != 23:
                failures.append(f"child {child}: unexpected wait result {(pid, status)}")
        except ChildProcessError as error:
            failures.append(f"child {child}: {error}")
    assert not failures, "native owner lost Python child statuses: " + "; ".join(failures)
    print("HOST-WAIT-AFTER OK", flush=True)


@pytest.mark.skipif(not hasattr(os, "posix_spawn"), reason="POSIX child ownership gate")
def test_native_parser_preserves_python_child_wait_ownership():
    result = subprocess.run(
        [sys.executable, str(Path(__file__).resolve())],
        capture_output=True, text=True, timeout=20, check=False,
    )
    # Python subprocess can mask ECHILD as returncode 0. Require the child
    # probe's completion receipt, not just the outer process exit code.
    assert "HOST-WAIT-BEFORE OK" in result.stdout, result.stderr
    assert "HOST-WAIT-AFTER OK" in result.stdout, result.stderr
    assert "HOST-STARTUP-RESOURCES OK" in result.stdout, result.stderr


if __name__ == "__main__":
    _probe()
