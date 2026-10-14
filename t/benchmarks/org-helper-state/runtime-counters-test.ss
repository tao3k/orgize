(import (only-in :std/test check check-exception test-case test-suite)
        (only-in "runtime-counters.ss" runtime-counter-vector? sum-runtime-counters
                 helper-observation-valid?))
(export runtime-counters-test)
(def runtime-counters-test
  (test-suite "Native helper runtime diagnostic projection"
    (test-case "timing absence and diagnostic counters stay distinct"
      (check (helper-observation-valid? 'timing 'disabled 'disabled 0 #f #f) => #t)
      (check (helper-observation-valid? 'diagnostic 'asp-run-inclusive
               'asp-callback-10-folds 20 '#(1 2 3 4) '#(5 6 7 8)) => #t)
      (check (helper-observation-valid? 'timing 'disabled 'disabled 0
               '#(0 0 0 0) #f) => #f)
      (check (helper-observation-valid? 'diagnostic 'asp-run-inclusive
               'asp-callback-10-folds 20 #f #f) => #f)
      (check (helper-observation-valid? 'diagnostic 'disabled 'disabled 20
               '#(1 2 3 4) '#(5 6 7 8)) => #f)
      (check (helper-observation-valid? 'timing 'disabled 'disabled 20 #f #f) => #f)
      (check (helper-observation-valid? 'unknown 'disabled 'disabled 0 #f #f) => #f))
    (test-case "aggregation preserves units and does not mutate samples"
      (let ((a (vector 7 11 13 1)) (b (vector 17 19 23 2)))
        (check (sum-runtime-counters (list a b)) => '#(24 30 36 3))
        (check a => '#(7 11 13 1))
        (check b => '#(17 19 23 2))
        (check (sum-runtime-counters '()) => '#(0 0 0 0))))
    (test-case "wrong shape, negative and inexact counters are rejected"
      (for-each
       (lambda (sample)
         (check (runtime-counter-vector? sample) => #f)
         (check-exception (sum-runtime-counters (list sample)) true))
       (list '() '#() '#(1 2 3) '#(1 2 3 4 5) '#(0 -1 0 0) '#(0 0.5 0 0)
             '#(0 #t 0 0) '#(0 0 +inf.0 0))))))
