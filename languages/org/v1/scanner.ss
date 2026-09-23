;;; -*- Gerbil -*-
;;; Org-owned contextual scanner model used by the AOT language pack.

(export +org-scanner-directives+ org-scan-lines)

(def +org-scanner-directives+
  '((block-begin . "#+begin_src")
    (block-end . "#+end_src")))

(def (horizontal-space? char)
  (or (char=? char #\space) (char=? char #\tab)))

(def (line-end-space? char)
  (or (horizontal-space? char)
      (char=? char #\return)
      (char=? char #\newline)))

;; Org block delimiters admit indentation, but END_SRC must occupy the rest
;; of its line; BEGIN_SRC may carry language and header arguments.
(def (directive-line? line directive closing?)
  (let* ((length (string-length line))
         (start (let loop ((index 0))
                  (if (and (< index length)
                           (horizontal-space? (string-ref line index)))
                    (loop (+ index 1))
                    index)))
         (after (+ start (string-length directive))))
    (and (<= after length)
         (string-ci=? (substring line start after) directive)
         (if closing?
           (let loop ((index after))
             (or (= index length)
                 (and (line-end-space? (string-ref line index))
                      (loop (+ index 1)))))
           (or (= after length)
               (line-end-space? (string-ref line after)))))))

(def (headline-line? line)
  (let loop ((index 0))
    (and (< index (string-length line))
         (if (char=? (string-ref line index) #\*)
           (loop (+ index 1))
           (and (> index 0)
                (char=? (string-ref line index) #\space))))))

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
                        (directive-line? line "#+end_src" #t)))
             (begin? (and (not in-source-block?)
                          (directive-line? line "#+begin_src" #f)))
             (terminal (cond (end? 'block-end)
                             (begin? 'block-begin)
                             (in-source-block? 'text)
                             ((headline-line? line) 'headline)
                             (else 'text))))
        (loop char-end (+ byte-start width)
              (cond (end? #f) (begin? #t) (else in-source-block?))
              (cons (list terminal byte-start (+ byte-start width)) rows))))))
