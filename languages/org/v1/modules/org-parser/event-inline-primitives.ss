;;; -*- Gerbil -*-
;;; Shared source-backed inline offsets and bounded marker predicates.

(export link-index inline-next pattern-end pattern-at?)

(def link-index '(line-index inline-byte-index))
(def inline-next `(line-step ,link-index))

(def (pattern-end from pattern)
  (let loop ((offset from) (remaining (string-length pattern)))
    (if (= remaining 0) offset
      (loop `(line-step ,offset) (- remaining 1)))))

(def (pattern-at? from pattern)
  (let ((bytes (string->utf8 pattern)))
    (unless (> (u8vector-length bytes) 0)
      (error "inline link marker must not be empty" pattern))
    (let loop ((offset from) (index 0) (predicate #f))
      (if (= index (u8vector-length bytes)) predicate
        (let (check `(line-byte-equal? ,offset ,(u8vector-ref bytes index)))
          (loop `(line-step ,offset) (+ index 1)
                (if predicate `(and ,predicate ,check) check)))))))

