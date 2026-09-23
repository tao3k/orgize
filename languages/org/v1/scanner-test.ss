;;; -*- Gerbil -*-
;;; Executable customer-scanner byte-range and context contracts.

(load "languages/org/v1/scanner.ss")

(def (assert-equal actual expected)
  (unless (equal? actual expected)
    (error "Org scanner contract mismatch" actual expected)))

(assert-equal (org-scan-lines "") '())
(assert-equal
 (org-scan-lines "#+BEGIN_SRC rust\r\n* not a headline\r\n#+END_SRC\r\n* Real\n")
 '((block-begin 0 18)
   (text 18 36)
   (block-end 36 47)
   (headline 47 54)))
(assert-equal
 (org-scan-lines "é\n** 标题\n")
 '((text 0 3) (headline 3 13)))
(assert-equal
 (org-scan-lines "#+begin_srcx\n*not headline\n")
 '((text 0 13) (text 13 27)))
(assert-equal
 (org-scan-lines "*\tNot a headline\n* Real\n")
 '((text 0 17) (headline 17 24)))
(assert-equal
 (org-scan-lines "  #+BEGIN_SRC rust\n  #+END_SRC trailing\n  #+END_SRC\n")
 '((block-begin 0 19) (text 19 40) (block-end 40 52)))
(displayln "Org scanner contracts OK")
