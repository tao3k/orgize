;;; -*- Gerbil -*-
;;; Org-owned contextual scanner model used by the AOT language pack.

(export +org-scanner-directives+ org-scan-lines)

(def +org-scanner-directives+
  '((block-begin . "#+begin_src")
    (block-end . "#+end_src")))

(def (directive-line? line directive)
  (let ((width (string-length directive)))
    (and (>= (string-length line) width)
         (string-ci=? (substring line 0 width) directive)
         (or (= (string-length line) width)
             (let ((next (string-ref line width)))
               (or (char=? next #\space)
                   (char=? next #\tab)
                   (char=? next #\return)
                   (char=? next #\newline)))))))

(def (headline-line? line)
  (let loop ((index 0))
    (and (< index (string-length line))
         (if (char=? (string-ref line index) #\*)
           (loop (+ index 1))
           (and (> index 0)
                (or (char=? (string-ref line index) #\space)
                    (char=? (string-ref line index) #\tab)))))))

(def (line-end source start)
  (let loop ((index start))
    (if (or (= index (string-length source))
            (char=? (string-ref source index) #\newline))
      (if (< index (string-length source)) (+ index 1) index)
      (loop (+ index 1)))))

;; Returns (terminal byte-start byte-end) rows. Every byte is covered exactly
;; once, including CRLF. A block body never reclassifies headline-looking text.
(def (org-scan-lines source)
  (let loop ((char-start 0) (byte-start 0) (in-source-block? #f) (rows '()))
    (if (= char-start (string-length source))
      (reverse rows)
      (let* ((char-end (line-end source char-start))
             (line (substring source char-start char-end))
             (width (u8vector-length (string->utf8 line)))
             (end? (and in-source-block?
                        (directive-line? line "#+end_src")))
             (begin? (and (not in-source-block?)
                          (directive-line? line "#+begin_src")))
             (terminal (cond (end? 'block-end)
                             (begin? 'block-begin)
                             (in-source-block? 'text)
                             ((headline-line? line) 'headline)
                             (else 'text))))
        (loop char-end (+ byte-start width)
              (cond (end? #f) (begin? #t) (else in-source-block?))
              (cons (list terminal byte-start (+ byte-start width)) rows))))))
