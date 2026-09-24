;;; -*- Gerbil -*-
;;; Structural AST events are asserted as values, never printed strings.

(import (only-in :std/test check test-case test-suite)
        (only-in "outline-events.ss"
                 org-headline-level parse-org-outline-events))
(export org-v1-outline-events-test)

(def org-v1-outline-events-test
  (test-suite "Org Scheme outline events"
    (test-case "nested and sibling headlines close at the right depth"
      (check (parse-org-outline-events
              "* Parent\n** Child\ntext\n* Peer\n")
             => '((start OrgFile)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 0 9) (finish)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 9 18) (finish)
                  (start OrgTextLine) (token TextLine 18 23) (finish)
                  (finish) (finish)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 23 30) (finish)
                  (finish) (finish))))
    (test-case "non-headline stars remain text and UTF-8 spans remain bytes"
      (check (parse-org-outline-events "*not a headline\n* α\n")
             => '((start OrgFile)
                  (start OrgTextLine) (token TextLine 0 16) (finish)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 16 21) (finish)
                  (finish) (finish))))
    (test-case "empty document has one closed root"
      (check (parse-org-outline-events "")
             => '((start OrgFile) (finish))))))
