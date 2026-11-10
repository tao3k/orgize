(import (only-in :std/test check test-case test-suite)
        (only-in "../org-helper-state.ss" helper-workload))
(export helper-state-test)

(def helper-state-test
  (test-suite "Org helper-state benchmark"
    (test-case "Repeated folds preserve the independent tape and reset helper state"
      (let-values (((fold expected) (helper-workload)))
        (check (fold) => expected)
        (check (fold) => expected)))))
