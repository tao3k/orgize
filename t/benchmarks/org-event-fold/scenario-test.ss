(import (only-in :std/test check test-case test-suite)
        (only-in :orgize/languages/org/modules/org-parser/event-runtime parse-org-native-events)
        (only-in "../../org-parser-test-support.ss" org-events-cover-source?))
(export event-fold-test)

(def event-fold-test
  (test-suite "Org event Fold benchmark"
    (test-case "A changing UTF-8 source retains complete byte coverage"
      (for-each
       (lambda (source)
         (let (events (parse-org-native-events source))
           (check (car events) => '(start OrgFile))
           (check (org-events-cover-source? source events) => #t)))
       '("* TODO Plan α\r\n- [[https://example.org][Link]]\r\n"
         "#+TITLE: Notes\n* DONE β\nA different document.\n")))))
