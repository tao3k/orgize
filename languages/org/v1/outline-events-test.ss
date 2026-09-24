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
             => '((start OrgFile) (finish))))
    (test-case "declared source block stays inside its section"
      (check (parse-org-outline-events
              "* Code\n#+BEGIN_SRC rust\nα\n#+END_SRC\n** Next\n")
             => '((start OrgFile)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 0 7) (finish)
                  (start OrgSourceBlock)
                  (token BlockBeginLine 7 24)
                  (token TextLine 24 27)
                  (token BlockEndLine 27 37)
                  (finish)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 37 45) (finish)
                  (finish) (finish) (finish))))
    (test-case "unclosed source block recovers before next headline"
      (check (parse-org-outline-events
              "* First\n#+begin_src rust\nbody\n** Next\n")
             => '((start OrgFile)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 0 8) (finish)
                  (start OrgTextLine) (token TextLine 8 25) (finish)
                  (start OrgTextLine) (token TextLine 25 30) (finish)
                  (start OrgSection)
                  (start OrgHeadline) (token HeadlineLine 30 38) (finish)
                  (finish) (finish) (finish))))))
