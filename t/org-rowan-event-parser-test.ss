;;; -*- Gerbil -*-
;;; Aggregate the independently owned Org event-parser test modules.
(import (only-in "org-rowan-inline-parser-test.ss"
                 org-v1-rowan-inline-parser-test)
        (only-in "org-rowan-structural-parser-test.ss"
                 org-v1-rowan-structural-parser-test)
        (only-in "org-rowan-table-container-parser-test.ss"
                 org-v1-rowan-table-container-parser-test))
;; gxtest discovers each exported *-test suite. Evaluating suite values inside
;; another test-suite only constructs an empty suite; it does not run them.
(export org-v1-rowan-inline-parser-test
        org-v1-rowan-structural-parser-test
        org-v1-rowan-table-container-parser-test)
