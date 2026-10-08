#!/usr/bin/env gxi
;;; Scheme adapter for the existing Orgize benchmark corpus, not Rust throughput.
(import (only-in :std/misc/ports read-all-as-string)
        (only-in :asp-gerbil-scheme/benchmark-api
                 benchmark-run/result benchmark-receipt-pass?
                 benchmark-fixture-contract-pass?)
        (only-in :gerbil-parser/src/runtime/identity sha256-text)
        (only-in "../../languages/org/native-event-runtime.ss" parse-org-native-events)
        (only-in "../org-parser-test-support.ss" org-events-cover-source?))
(export main)

;; Untimed semantic observations, not a dynamic predicate profiler.
(def (entity-result-summary source events)
  (let ((bytes (string->utf8 source)) (named 0) (spaces 0))
    (for-each
     (lambda (event)
       (when (and (eq? (car event) 'token) (eq? (cadr event) 'EntityName))
         (let ((begin (caddr event)) (end (cadddr event)))
           (unless (and (< begin end) (<= end (u8vector-length bytes)))
             (error "invalid admitted entity name span" event))
           (if (= (u8vector-ref bytes begin) 95)
             (set! spaces (+ spaces 1))
             (set! named (+ named 1))))))
     events)
    `((namedEntityResults . ,named) (spaceEntityResults . ,spaces)
      (namePredicateEvaluations . unmeasured))))

(def (main label fixture contract output)
  (displayln "ORG-EVENT-BENCHMARK-BEGIN " label " " fixture)
  (force-output)
  (let* ((configuration (call-with-input-file contract read))
         (source (call-with-input-file fixture read-all-as-string)))
    (unless (benchmark-fixture-contract-pass? configuration)
      (error "invalid ASP benchmark fixture" configuration))
    (when (and (not (equal? output "")) (file-exists? output))
      (error "refuse to overwrite benchmark receipt" output))
    (displayln "ORG-EVENT-BENCHMARK-SEMANTIC-BEGIN")
    (force-output)
    (let (expected (parse-org-native-events source))
      (unless (org-events-cover-source? source expected)
        (error "benchmark events do not partition the source" fixture))
      (displayln "ORG-EVENT-BENCHMARK-SAMPLING-BEGIN")
      (force-output)
      ;; ASP owns sampling, GC, timing, statistics and p95 admission.
      ;; Check the returned admitted result outside timing; this is not a check
      ;; of every intermediate timed operation.
      (let-values (((measurement result)
                    (benchmark-run/result
                     configuration (lambda () (parse-org-native-events source)))))
        (unless (equal? result expected)
          (error "benchmark event semantics changed" label fixture))
        (let ((receipt
               `((schema . orgize.scheme-event-fold.asp.v1)
                 (label . ,label)
                 (fixture . ,fixture)
                 (contract . ,configuration)
                 (libraries . ,(getenv "GERBIL_LOADPATH" ""))
                 (sourceBytes . ,(u8vector-length (string->utf8 source)))
                 (sourceIdentity . ,(sha256-text source))
                 (eventIdentity . ,(sha256-text
                                    (call-with-output-string
                                     (lambda (port) (write expected port)))))
                 (entityResultSummary . ,(entity-result-summary source expected))
                 (benchmark . ,measurement))))
          (unless (equal? output "")
            (call-with-output-file output
              (lambda (port) (write receipt port) (newline port))))
          (write receipt)
          (newline)
          (force-output)
          (unless (benchmark-receipt-pass? measurement)
            (error "ASP benchmark admission failed; diagnostic receipt preserved"
                   label output))))
      (displayln "ORG-EVENT-BENCHMARK-OK"))))
