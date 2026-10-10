(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :orgize/bindings/c/runtime-statistics
                 runtime-statistics-snapshot runtime-statistics-delta)
        (only-in :orgize/t/clock/reference reference-process-cpu-ns reference-spin))
(export runtime-statistics-test)

(def (sample)
  (make-f64vector 20 0.))

(def runtime-statistics-test
  (test-suite "Native runtime statistics owner qualification"
    (test-case "typed projection uses seconds and counts distinctly"
      (let ((begin (sample)) (end (sample)))
        (for-each (lambda (entry) (f64vector-set! end (car entry) (cdr entry)))
                  '((0 . 1.) (1 . 2.) (3 . 0.25) (4 . 0.5)
                    (5 . 1.5) (6 . 2.)))
        (check (runtime-statistics-delta begin end)
               => '#(3000000000 750000000 1500000000 2))))
    (test-case "unused allocation counter cannot poison CPU and GC projection"
      (let ((begin (sample)) (end (sample)))
        (f64vector-set! begin 7 4096.)
        (f64vector-set! end 7 0.)
        (check (runtime-statistics-delta begin end) => '#(0 0 0 0))))
    (test-case "invalid layout, counters, and regression fail closed"
      (check-exception (runtime-statistics-delta (vector) (sample)) true)
      (check-exception (runtime-statistics-delta (make-f64vector 19 0.) (sample)) true)
      (for-each
       (lambda (value)
         (let ((end (sample)))
           (f64vector-set! end 6 value)
           (check-exception (runtime-statistics-delta (sample) end) true)))
       '(-1. 0.5 +inf.0 +nan.0 9007199254740992.))
      (for-each
       (lambda (index)
         (let ((begin (sample)))
           (f64vector-set! begin index 1.)
           (check-exception (runtime-statistics-delta begin (sample)) true)))
       '(0 1 3 4 5 6))
      (let ((end (sample)))
        (f64vector-set! end 0 20000000000.)
        (check-exception (runtime-statistics-delta (sample) end) true)))
    (test-case "real process CPU brackets the independent OS clock"
      (let* ((begin (runtime-statistics-snapshot))
             (cpu-begin (reference-process-cpu-ns))
             (result (reference-spin 16000000))
             (cpu-end (reference-process-cpu-ns))
             (end (runtime-statistics-snapshot))
             (delta (runtime-statistics-delta begin end)))
        (check result => (quotient (* 16000000 15999999) 2))
        (check (> (vector-ref delta 0) 0) => #t)
        ;; The process-times owner uses microsecond counters on this platform.
        (check (<= (- cpu-end cpu-begin) (+ (vector-ref delta 0) 2000)) => #t)
        (displayln "NATIVE-RUNTIME-STATS cpu-work delta=" delta
                   " reference_process_ns=" (- cpu-end cpu-begin))
        (force-output)))
    (test-case "explicit test-only GC advances the VM collection counter"
      (let ((begin (runtime-statistics-snapshot)))
        ;; Only the qualification test forces GC, never the diagnostic helper.
        (##gc)
        (let* ((end (runtime-statistics-snapshot))
               (delta (runtime-statistics-delta begin end)))
          (check (>= (vector-ref delta 3) 1) => #t)
          (check (> (vector-ref delta 1) 0) => #t)
          (check (> (vector-ref delta 2) 0) => #t)
          (displayln "NATIVE-RUNTIME-STATS forced-gc delta=" delta)
          (force-output))))))
