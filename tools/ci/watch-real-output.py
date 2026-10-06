# SPDX-FileCopyrightText: 2026 tao3k team and Contributors
# SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
"""Fail a child process after five seconds without genuine output."""

import os
import selectors
import signal
import subprocess
import sys


child = subprocess.Popen(
    sys.argv[1:], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True
)
print(f"CHILD-START pid={child.pid}", flush=True)
try:
    with selectors.DefaultSelector() as ready:
        ready.register(child.stdout, selectors.EVENT_READ)
        while True:
            if not ready.select(timeout=5):
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
                print("FAIL: five seconds without real source qualification output", flush=True)
                sys.exit(124)
            data = os.read(child.stdout.fileno(), 65536)
            if not data:
                break
            sys.stdout.buffer.write(data)
            sys.stdout.flush()
    sys.exit(child.wait())
finally:
    if child.poll() is None:
        os.killpg(child.pid, signal.SIGKILL)
        child.wait()
