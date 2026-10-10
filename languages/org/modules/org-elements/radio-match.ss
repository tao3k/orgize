;;; -*- Gerbil -*-
;;; Document-local radio target matching.  The POO strategy is executable in
;;; Scheme and is also the sole source of the AOT matcher IR.

(import (only-in :clan/poo/object .ref)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot string-prefix?)
        (only-in "types.ss" org-source-match-strategy?)
        (only-in "objects.ss" make-org-source-match-strategy))
(export org-radio-match-strategy org-next-radio-match org-radio-matches)

(def org-radio-match-strategy (make-org-source-match-strategy "_-"))

(def (utf8-width char)
  (u8vector-length (string->utf8 (string char))))

(def (word-char? strategy char)
  (and char
       (or (and (.ref strategy 'boundary-unicode-alphanumeric)
                (or (char-alphabetic? char) (char-numeric? char)))
           (member char (string->list (.ref strategy 'boundary-extra))))))

(def (target-at? strategy source offset target before)
  (let ((end (+ offset (string-length target)))
        (size (string-length source)))
    (and (> (string-length target) 0)
         (not (word-char? strategy before))
         (<= end size)
         (string-prefix? (substring source offset size) target)
         (not (word-char? strategy
                          (and (< end size) (string-ref source end)))))))

(def (best-target strategy source offset before targets)
  (let loop ((rest targets) (index 0) (best #f))
    (if (null? rest) best
      (let (target (car rest))
        (loop (cdr rest) (+ index 1)
              (if (and (target-at? strategy source offset target before)
                       (or (not best)
                           (> (u8vector-length (string->utf8 target))
                              (u8vector-length (string->utf8 (cdr best))))))
                (cons index target) best))))))

;; Returns #(byte-start byte-end target-index), or #f.  Character iteration
;; carries its UTF-8 byte offset once; no prefix is re-encoded on every step.
(def (org-next-radio-match strategy source cursor targets)
  (unless (and (org-source-match-strategy? strategy) (string? source)
               (integer? cursor) (<= 0 cursor) (list? targets))
    (error "invalid Org radio match input"))
  (let loop ((chars (string->list source)) (character-offset 0)
             (byte-offset 0) (previous-byte-offset 0) (before #f))
    (if (null? chars) #f
      (if (and (< previous-byte-offset cursor) (< cursor byte-offset))
        #f
        (let* ((current (car chars))
               (best (and (>= byte-offset cursor)
                          (best-target strategy source character-offset
                                       before targets))))
          (if best
            (vector byte-offset
                    (+ byte-offset
                       (u8vector-length (string->utf8 (cdr best))))
                    (car best))
            (loop (cdr chars) (+ character-offset 1)
                  (+ byte-offset (utf8-width current))
                  byte-offset current)))))))

;; Pre-index targets by first character; one UTF-8 source walk, no suffix copy.
(def (org-radio-matches source targets)
  (let ((index (make-hash-table)) (end (string-length source)))
    (let loop ((rest targets) (id 0))
      (unless (null? rest)
        (let ((target (car rest)))
          (unless (string=? target "")
            (let (key (string-ref target 0))
              (hash-put! index key (cons (vector id target (string-length target)
                                                (u8vector-length (string->utf8 target)))
                                        (or (hash-get index key) '())))))
          (loop (cdr rest) (+ id 1)))))
    (let walk ((at 0) (byte 0) (before #f) (out '()))
      (if (= at end) (reverse out)
        (let* ((ch (string-ref source at))
               (best
                (and (not (word-char? org-radio-match-strategy before))
                     (foldl
                      (lambda (target best)
                        (let (stop (+ at (vector-ref target 2)))
                          (if (and (<= stop end)
                                   (not (word-char? org-radio-match-strategy
                                                    (and (< stop end) (string-ref source stop))))
                                   (let same ((offset 0))
                                     (or (= offset (vector-ref target 2))
                                         (and (char=? (string-ref source (+ at offset))
                                                      (string-ref (vector-ref target 1) offset))
                                              (same (+ offset 1)))))
                                   (or (not best) (> (vector-ref target 3) (vector-ref best 3))
                                       (and (= (vector-ref target 3) (vector-ref best 3))
                                            (< (vector-ref target 0) (vector-ref best 0)))))
                            target best)))
                      #f (or (hash-get index ch) '())))))
          (walk (+ at 1) (+ byte (utf8-width ch)) ch
                (if best
                  (cons (map number->string
                             (list byte (+ byte (vector-ref best 3)) (vector-ref best 0))) out)
                  out)))))))
