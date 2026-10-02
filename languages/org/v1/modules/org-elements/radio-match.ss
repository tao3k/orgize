;;; -*- Gerbil -*-
;;; Document-local radio target matching.  The POO strategy is executable in
;;; Scheme and is also the sole source of the AOT matcher IR.

(import (only-in :clan/poo/object .ref)
        (only-in :std/encoding/json json->string)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot string-prefix?)
        (only-in "types.ss" org-source-match-strategy?)
        (only-in "objects.ss" make-org-source-match-strategy))
(export org-radio-match-strategy org-next-radio-match
        org-radio-match-ir-json write-org-radio-match-ir)

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

(def (org-radio-match-ir-json strategy)
  (unless (org-source-match-strategy? strategy)
    (error "Org radio matcher requires a POO strategy" strategy))
  (json->string
   (hash (schema "gerbil-scheme-rust.source-match-ir.v1")
         (name "org_radio_next_match")
         (scan "utf8_character_boundaries")
         (candidate "exact_target_prefix")
         (boundary
          (hash (unicode_alphanumeric
                 (.ref strategy 'boundary-unicode-alphanumeric))
                (extra_word_characters (.ref strategy 'boundary-extra))))
         (winner "longest_then_first"))
   sort-keys: #t))

(def (write-org-radio-match-ir path strategy)
  (call-with-output-file path
    (lambda (port)
      (write-string (org-radio-match-ir-json strategy) port))))
