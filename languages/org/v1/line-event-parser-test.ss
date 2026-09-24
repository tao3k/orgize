;;; -*- Gerbil -*-
;;; The exact Scheme function lowered to Rust retains UTF-8 byte ranges.

(import (only-in :std/test check test-case test-suite)
        (only-in "line-event-parser.ss" parse-org-line-events))
(export org-v1-line-event-parser-test)

(def org-v1-line-event-parser-test
  (test-suite "Org-owned line event algorithm"
    (test-case "headline and text preserve UTF-8 byte offsets"
      (check (parse-org-line-events "* α\r\nbody\n")
             => '((start OrgFile)
                  (start OrgHeadline) (token HeadlineLine 0 6) (finish)
                  (start OrgTextLine) (token TextLine 6 11) (finish)
                  (finish))))
    (test-case "empty source closes the document"
      (check (parse-org-line-events "")
             => '((start OrgFile) (finish))))))
