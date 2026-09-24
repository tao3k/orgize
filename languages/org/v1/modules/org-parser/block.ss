;;; -*- Gerbil -*-
;;; Org block openings and typed header tokens, including named drawers.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 block-line-opening block-line-opening-mode
                 block-line-closing block-line-case-insensitive
                 block-line-indent block-line-header block-line-begin-token
                 block-header-argument-token block-header-trivia-token)
        (only-in "funs.ss"
                 bytes-match? line-content-end line-marker?
                 skip-horizontal scan-word token-if-nonempty))
(export org-block-opening? org-block-opening-events)

(def (ascii-letter? byte)
  (or (<= 65 byte 90) (<= 97 byte 122)))

(def (ascii-name-byte? byte)
  (or (ascii-letter? byte)
      (<= 48 byte 57)
      (memv byte '(45 95))))

(def (horizontal-trivia? byte)
  (memv byte '(9 32)))

(def (line-trivia-only? bytes start end)
  (let loop ((offset start))
    (or (= offset end)
        (and (horizontal-trivia? (u8vector-ref bytes offset))
             (loop (+ offset 1))))))

(def (closing-line? bytes span rule)
  (let* ((end (line-content-end bytes span))
         (start (if (block-line-indent rule)
                  (skip-horizontal bytes (car span) end)
                  (car span)))
         (size (u8vector-length
                (string->utf8 (block-line-closing rule)))))
    (and (bytes-match? bytes start end (block-line-closing rule)
                       (block-line-case-insensitive rule))
         (line-trivia-only? bytes (+ start size) end))))

(def (named-drawer-name bytes span rule)
  (let* ((end (line-content-end bytes span))
         (start (if (block-line-indent rule)
                  (skip-horizontal bytes (car span) end)
                  (car span)))
         (marker (string->utf8 (block-line-opening rule)))
         (marker-size (u8vector-length marker))
         (name-start (+ start marker-size)))
    (and (= marker-size 1)
         (< name-start end)
         (not (closing-line? bytes span rule))
         (bytes-match? bytes start end (block-line-opening rule)
                       (block-line-case-insensitive rule))
         (ascii-letter? (u8vector-ref bytes name-start))
         (let loop ((offset (+ name-start 1)))
           (if (and (< offset end)
                    (ascii-name-byte? (u8vector-ref bytes offset)))
             (loop (+ offset 1))
             (and (< offset end)
                  (= (u8vector-ref bytes offset)
                     (u8vector-ref marker 0))
                  (line-trivia-only? bytes (+ offset 1) end)
                  (cons name-start offset)))))))

(def (required-name? bytes span rule)
  (and (line-marker? bytes span (block-line-opening rule)
                     (block-line-case-insensitive rule)
                     (block-line-indent rule))
       (let* ((end (line-content-end bytes span))
              (indent (if (block-line-indent rule)
                        (skip-horizontal bytes (car span) end)
                        (car span)))
              (prefix-end (+ indent
                             (u8vector-length
                              (string->utf8 (block-line-opening rule)))))
              (start (skip-horizontal bytes prefix-end end))
              (name-end (scan-word bytes start end)))
         (and (< start name-end)
              (ascii-letter? (u8vector-ref bytes start))
              (let loop ((offset (+ start 1)))
                (or (= offset name-end)
                    (and (ascii-name-byte? (u8vector-ref bytes offset))
                         (loop (+ offset 1)))))))))

(def (org-block-opening? bytes span rule)
  (case (block-line-opening-mode rule)
    ((named-delimited) (named-drawer-name bytes span rule))
    ((required-named-argument) (required-name? bytes span rule))
    (else (line-marker? bytes span (block-line-opening rule)
                        (block-line-case-insensitive rule)
                        (block-line-indent rule)))))

(def (org-block-opening-events bytes span rule)
  (let* ((start (car span))
         (end (cdr span))
         (header (block-line-header rule))
         (named (and (eq? (block-line-opening-mode rule)
                          'named-delimited)
                     (named-drawer-name bytes span rule))))
    (if named
      (list (list 'token (block-line-begin-token rule)
                  start (car named))
            (list 'token (block-header-argument-token header)
                  (car named) (cdr named))
            (list 'token (block-header-trivia-token header)
                  (cdr named) end))
      (if (not header)
        (list (list 'token (block-line-begin-token rule) start end))
        (let* ((content-end (line-content-end bytes span))
               (indent (if (block-line-indent rule)
                         (skip-horizontal bytes start content-end) start))
               (prefix-end (+ indent
                              (u8vector-length
                               (string->utf8 (block-line-opening rule)))))
               (argument-start (skip-horizontal bytes prefix-end content-end))
               (argument-end (scan-word bytes argument-start content-end)))
          (append
           (list (list 'token (block-line-begin-token rule)
                       start prefix-end))
           (token-if-nonempty (block-header-trivia-token header)
                              prefix-end argument-start)
           (token-if-nonempty (block-header-argument-token header)
                              argument-start argument-end)
           (token-if-nonempty (block-header-trivia-token header)
                              argument-end end)))))))
