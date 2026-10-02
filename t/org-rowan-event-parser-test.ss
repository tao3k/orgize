;;; -*- Gerbil -*-
;;; Aggregate the independently owned Org event-parser test modules.
(import (only-in :std/test test-suite)
        (only-in "org-rowan-inline-parser-test.ss"
                 org-v1-rowan-inline-parser-test)
        (only-in "org-rowan-structural-parser-test.ss"
                 org-v1-rowan-structural-parser-test)
        (only-in "org-rowan-table-container-parser-test.ss"
                 org-v1-rowan-table-container-parser-test))
(export org-v1-rowan-event-parser-test)

(def org-v1-rowan-event-parser-test
  (test-suite "Org contextual Rowan event AOT"
    org-v1-rowan-inline-parser-test
    org-v1-rowan-structural-parser-test
    org-v1-rowan-table-container-parser-test))
