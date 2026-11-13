#!/usr/bin/env gxi
(import (only-in :asp-gerbil-scheme/benchmark-api
                 benchmark-run/result benchmark-receipt-pass?)
        (only-in :gerbil-parser/rust-runtime-event-support run-event-fold)
        (only-in :orgize/languages/org/modules/org-parser/entity-names
                 org-entity-names))
(export main name-set-fold)

(def source
  (string-append "AA\nAlpha\nbeta\nzeta\nzz\nα\nalphaX\nalpha\nA\n\n"
                 "AA\nAlpha\nbeta\nzeta\nzz\nα\nalphaX\nalpha\nA\n\n"))
(def forms
  `((if (line-bytes-in-set? start (line-content-end) ,org-entity-names)
        ((start-node Heading) (token Line start end) (finish-node))
        ((start-node Text) (token Line start end) (finish-node)))))
(def (name-set-fold)
  (run-event-fold source 'Document '() forms '()))

(def (main label contract output)
  (when (file-exists? output) (error "refuse to overwrite benchmark receipt" output))
  (let* ((fixture (call-with-input-file contract read))
         (expected (name-set-fold)))
    (displayln "ORG-NAME-SET-BENCHMARK-BEGIN " label)
    (force-output)
    (unless (= (cdr (assq 'batchOperations fixture)) 100)
      (error "name-set scenario requires exactly 100 complete folds" fixture))
    (let-values (((measurement result)
                  (benchmark-run/result fixture
                    (lambda ()
                      (let loop ((remaining 100) (last #f))
                        (if (= remaining 0) last
                          (loop (- remaining 1) (name-set-fold))))))))
      (unless (equal? result expected)
        (error "name-set benchmark event semantics changed" label))
      (let (receipt `((schema . orgize.scheme-name-set.asp.v1)
                     (label . ,label) (source . ,source) (names . ,org-entity-names)
                     (events . ,result) (contract . ,fixture)
                     (libraries . ,(getenv "GERBIL_LOADPATH" ""))
                     (benchmark . ,measurement)))
        (call-with-output-file output (lambda (port) (write receipt port) (newline port)))
        ;; The full matched input and event result live in the native receipt.
        (write measurement) (newline)
        ;; Preserve failed diagnostic measurements, never promote them to pass.
        (unless (benchmark-receipt-pass? measurement)
          (error "name-set benchmark admission failed; receipt preserved" output))
        (displayln "ORG-NAME-SET-BENCHMARK-OK")))))
