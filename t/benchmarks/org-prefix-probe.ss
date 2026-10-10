#!/usr/bin/env gxi
(import (only-in :asp-gerbil-scheme/benchmark-api
                 micro-kernel-run/result micro-kernel-receipt-pass?)
        (only-in :gerbil-parser/src/compiler/event-source-lines line-starts-with?))
(export main)

(def (main label contract output)
  (when (file-exists? output) (error "refuse to overwrite benchmark receipt" output))
  (let ((fixture (call-with-input-file contract read))
        (line "#+BEGIN_QUOTE\n")
        (prefix "#+"))
    (displayln "ORG-PREFIX-BENCHMARK-BEGIN " label)
    (force-output)
    (let-values (((measurement result)
                  (micro-kernel-run/result fixture
                    (lambda () (line-starts-with? line prefix)))))
      (unless (and (eq? result #t) (micro-kernel-receipt-pass? measurement))
        (error "prefix benchmark admission failed" measurement result))
      (let (receipt
            `((schema . orgize.scheme-prefix-probe.asp.v1)
              (label . ,label)
              (libraries . ,(getenv "GERBIL_LOADPATH" ""))
              (line . ,line) (prefix . ,prefix)
              (contract . ,fixture) (benchmark . ,measurement)))
        (call-with-output-file output
          (lambda (port) (write receipt port) (newline port)))
        (write receipt)
        (newline)
        (displayln "ORG-PREFIX-BENCHMARK-OK")))))
