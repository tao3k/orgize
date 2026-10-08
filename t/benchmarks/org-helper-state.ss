#!/usr/bin/env gxi
(import (only-in :asp-gerbil-scheme/benchmark-api
                 benchmark-run/result benchmark-receipt-pass?)
        (only-in :gerbil-parser/src/compiler/event-fold-runtime run-event-fold)
        (only-in :orgize/bindings/c/native-runtime-statistics
                 native-runtime-statistics-snapshot native-runtime-statistics-delta)
        (only-in "org-helper-state/runtime-counters.ss" sum-runtime-counters))
(export main)

(def (run-helper-batch operation)
  (let loop ((remaining 10) (last #f))
    (if (= remaining 0) last
      (loop (- remaining 1) (operation)))))

(def (main label contract output mode)
  (unless (member mode '("timing" "diagnostic"))
    (error "unsupported helper benchmark mode" mode))
  (when (file-exists? output) (error "refuse to overwrite benchmark receipt" output))
  (let* ((fixture (call-with-input-file contract read))
         (source (apply string-append (make-list 64 "helper α\n")))
         (defaults
          (let loop ((n 0) (entries '((value 0) (done #f) (frames (uint-stack)))))
            (if (= n 32) entries
              (loop (+ n 1)
                    (cons (list (string->symbol (string-append "kept-" (number->string n))) n)
                          entries)))))
         (forms '((call-source-helper outer start end ((uint 7)))))
         (helpers
          `((outer ,defaults
                   ((call-source-helper inner start end ((state value))))
                   (value))
            (inner ,defaults
                   ((if (uint-equal? (state value) (uint 7))
                        ((token Line start end)) ((start-node WrongArgument)))
                    (if (state done) ((start-node LeakedDefault)) ())
                    (set-bool done (bool #t))
                    (push-frame frames (uint 7)))
                   (value))))
         (operation (lambda () (run-event-fold source 'Document '() forms '() helpers)))
         (expected
          (cons '(start Document)
                (let loop ((n 0))
                  (if (= n 64) '((finish))
                    (cons (list 'token 'Line (* n 10) (* (+ n 1) 10))
                          (loop (+ n 1))))))))
    (unless (equal? (operation) expected)
      (error "helper scenario admission differs from independent byte tape"))
    (unless (and (= (cdr (assq 'batchOperations fixture)) 10)
                 (= (cdr (assq 'sourceLines fixture)) 64)
                 (= (cdr (assq 'helperCallsPerFold fixture)) 128))
      (error "helper scenario requires declared 10-fold/64-line/128-call units"))
    (displayln "ORG-HELPER-STATE-BENCHMARK-BEGIN " label)
    (force-output)
    ;; One native process/VM envelope around ASP's existing run. This includes
    ;; framework work, is not exclusive helper cost, and owns no percentiles.
    ;; Snapshot allocation may contribute to GC. This wrapper forces no GC;
    ;; ASP's own native GC precondition is included in the observed interval.
    (let ((stats-begin (and (equal? mode "diagnostic")
                           (native-runtime-statistics-snapshot)))
          (workload-samples '()))
      (let-values (((measurement result)
                  (benchmark-run/result fixture
                    (if (equal? mode "timing")
                      ;; Select outside the timed callback: no CPU/GC probes.
                      (lambda () (run-helper-batch operation))
                      (lambda ()
                      ;; Observe the existing ASP callback, not its GC
                      ;; precondition. Snapshot overhead remains visible.
                      (let* ((begin (native-runtime-statistics-snapshot))
                             (result (run-helper-batch operation))
                             (end (native-runtime-statistics-snapshot)))
                        (set! workload-samples
                              (cons (native-runtime-statistics-delta begin end)
                                    workload-samples))
                        result))))))
        (let (runtime-counters
              (and stats-begin
                   (native-runtime-statistics-delta
                    stats-begin (native-runtime-statistics-snapshot))))
          (unless (equal? result expected)
            (error "helper benchmark event semantics changed" label))
          (unless (= (length workload-samples) (if stats-begin 20 0))
            (error "unexpected ASP workload callback count" (length workload-samples)))
          (let (receipt `((schema . orgize.scheme-helper-state.asp.v3)
                         (mode . ,(if stats-begin 'diagnostic 'timing))
                         (label . ,label) (source . ,source)
                         (initial . ,defaults) (forms . ,forms) (helpers . ,helpers)
                         (events . ,result) (contract . ,fixture)
                         (diagnosticScope . ,(if stats-begin 'asp-run-inclusive 'disabled))
                         ;; ns, ns, ns, count; not allocation bytes or sample stats.
                         (runtimeCounters . ,runtime-counters)
                         (workloadScope . ,(if stats-begin 'asp-callback-10-folds 'disabled))
                         (workloadCallbacks . ,(length workload-samples))
                         (workloadCounters . ,(and stats-begin
                                                  (sum-runtime-counters workload-samples)))
                         (libraries . ,(getenv "GERBIL_LOADPATH" ""))
                         (benchmark . ,measurement)))
            (call-with-output-file output (lambda (port) (write receipt port) (newline port)))
            (write measurement) (newline)
            (unless (benchmark-receipt-pass? measurement)
              (error "helper benchmark admission failed; receipt preserved" output))
            (displayln "ORG-HELPER-STATE-BENCHMARK-OK")))))))
