;;; -*- Gerbil -*-
;;; Exact Scheme-side expectations for the document-local AOT matcher.

(import (only-in :std/test check check-exception test-case test-suite)
        (only-in "../languages/org/modules/org-elements/types.ss" org-source-match-strategy?)
        (only-in "../languages/org/modules/org-elements/objects.ss" make-org-source-match-strategy)
        (only-in "../languages/org/modules/org-elements/radio-match.ss"
                 org-radio-match-strategy org-next-radio-match org-radio-matches))
(export org-radio-match-test)

(defrules check-radio-match ()
  ((_ source cursor targets expected)
   (check (org-next-radio-match org-radio-match-strategy
                                source cursor targets)
          => expected)))

(def org-radio-match-test
  (test-suite "Org radio matcher POO and AOT strategy"
    (test-case "the strategy is admitted and executes a native matching plan"
      (check (org-source-match-strategy? org-radio-match-strategy) => #t)
      (check (org-radio-matches "é Alpha Beta Alpha" '("Alpha" "Alpha Beta" "Alpha"))
             => '(("3" "13" "1") ("14" "19" "0")))
      (check-exception (make-org-source-match-strategy #f) true))
    (test-case "longest target wins; duplicates keep first declaration"
      (check-radio-match "Alpha Beta Alpha Alphabet" 0
                         '("Alpha" "Alpha Beta" "Alpha") #(0 10 1))
      (check-radio-match "Alpha Beta Alpha Alphabet" 10
                         '("Alpha" "Alpha Beta" "Alpha") #(11 16 0))
      (check-radio-match "Alpha" 0 '("Alpha" "Alpha") #(0 5 0)))
    (test-case "word boundaries exclude suffixes and hyphenated words"
      (check-radio-match "Alpha-Beta" 0 '("Alpha") #f)
      (check-radio-match "Alphabet" 0 '("Alpha") #f)
      (check-radio-match "Alpha, not Alpha." 5 '("Alpha") #(11 16 0)))
    (test-case "UTF-8 source offsets remain byte-based"
      (check-radio-match "é Alpha" 0 '("Alpha") #(3 8 0))
      (check-radio-match "é Alpha" 1 '("Alpha") #f)
      (check-radio-match "é Alpha" 3 '("Alpha") #(3 8 0)))
    (test-case "marked target text remains exact and case-sensitive"
      (check-radio-match "*Radio* Radio" 0 '("*Radio*") #(0 7 0))
      (check-radio-match "radio" 0 '("Radio") #f))))
