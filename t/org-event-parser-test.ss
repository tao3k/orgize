;;; -*- Gerbil -*-
;;; Aggregate the independently owned Org event-parser test modules.
(import (only-in "org-inline-parser-test.ss"
                 org-inline-parser-test)
        (only-in "org-structural-parser-test.ss"
                 org-structural-parser-test)
        (only-in "org-table-container-parser-test.ss"
                 org-table-container-parser-test))
;; gxtest discovers each exported *-test suite. Evaluating suite values inside
;; another test-suite only constructs an empty suite; it does not run them.
(export org-inline-parser-test
        org-structural-parser-test
        org-table-container-parser-test)
