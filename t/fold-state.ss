;;; -*- Gerbil -*-
;;; Native standard suites over the actual parser-owned persistent operation.
(import (only-in :std/test test-suite test-case check)
        (only-in :std/test/base TestHarness TestConfig TestModule test-run! test-result-ok?)
        (only-in :gerbil-parser/src/compiler/event-fold-state fold-update-state)
        (only-in :gerbil-parser/t/event-fold-state-test
                 event-fold-state-test reference-update)
        (only-in :orgize/bindings/c/clock native-monotonic-ns native-thread-cpu-ns))
(export main)

(def (observe-update update states name alternating?)
  (let* ((wall (native-monotonic-ns))
         (cpu (native-thread-cpu-ns))
         (result
          (let loop ((index 0) (state states))
            (if (= index 10000) state
              (loop (+ index 1)
                    (update state name (and alternating? (even? index))))))))
    (values result (- (native-monotonic-ns) wall)
            (- (native-thread-cpu-ns) cpu))))

(def suite
  (test-suite "Persistent event fold native controls"
    (test-case "five fixed native CPU and wall control pairs"
      (let* ((names (map (lambda (index)
                          (string->symbol (string-append "state-" (number->string index))))
                        (iota 64)))
             (states (map (lambda (name) (cons name #f)) names)))
        (for-each
         (lambda (sample)
           (for-each
            (lambda (case)
              (let ((name (list-ref names (car case))) (alternating? (cadr case)))
                (let-values (((reference rw rc)
                              (observe-update reference-update states name alternating?))
                             ((candidate cw cc)
                              (observe-update fold-update-state states name alternating?)))
                  (check candidate => reference)
                  (displayln "FOLD-STATE-CONTROL sample=" sample
                             " slot=" (car case) " alternating=" alternating?
                             " states=64 updates=10000 reference_wall_ns=" rw
                             " candidate_wall_ns=" cw " reference_cpu_ns=" rc
                             " candidate_cpu_ns=" cc)
                  (force-output))))
            '((0 #f) (0 #t) (31 #t) (63 #t))))
         (iota 5))))))

(def (main . args)
  (unless (or (null? args) (equal? args '("--failure-control")))
    (error "fold state control accepts no filters" args))
  (let (result
        (test-run! (TestHarness "native fold state"
                               (TestConfig verbosity: 5 capture-output?: #f)
                               (list (TestModule "native fold state"
                                                 (if (null? args)
                                                   (list event-fold-state-test suite)
                                                   (list (test-suite "failure exit control"
                                                           (test-case "intentional failure"
                                                             (check #f => #t)))))
                                                 '() void void)))))
    (if (test-result-ok? result)
      (begin (displayln "OK") (exit 0))
      (begin (displayln "FAILED") (exit 42)))))
