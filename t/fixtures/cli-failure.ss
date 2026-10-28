;;; Negative native CLI control; excluded from ordinary directory discovery.
(import (only-in :std/test test-suite test-case check))
(export cli-failure-test)
(def cli-failure-test
  (test-suite "native CLI failure status"
    (test-case "intentional failed assertion" (check #f => #t))))
