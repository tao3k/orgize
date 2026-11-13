;;; Discover benchmark scenario suites through Gerbil's standard test entry.
(import (only-in "t/benchmarks/org-prefix-probe/scenario-test.ss" prefix-probe-test)
        (only-in "t/benchmarks/org-name-set/scenario-test.ss" name-set-test)
        (only-in "t/benchmarks/org-helper-state/scenario-test.ss" helper-state-test)
        (only-in "t/benchmarks/org-helper-state/runtime-counters-test.ss" runtime-counters-test)
        (only-in "t/benchmarks/org-event-fold/scenario-test.ss" event-fold-test))
(export prefix-probe-test name-set-test helper-state-test runtime-counters-test event-fold-test)
