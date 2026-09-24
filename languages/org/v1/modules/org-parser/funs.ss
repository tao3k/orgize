;;; -*- Gerbil -*-
;;; Shared source-backed byte functions for Org parser strategies.
;;; Byte offsets are retained for Rowan; decoding per line would lose them.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-key-lines key-line-prefix key-line-keys
                 key-line-separator key-line-case-insensitive
                 key-line-indent key-line-node key-line-key-token
                 key-line-value-token key-line-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure))

(export org-headline-level token-if-nonempty
        skip-horizontal scan-word strip-trailing-space
        scan-key bytes-match? line-content-end line-marker?
        matching-key-line emit-key-line)

(def (org-headline-level bytes start end)
  (let loop ((offset start) (level 0))
    (cond
     ((>= offset end) 0)
     ((= (u8vector-ref bytes offset) 42)
      (loop (+ offset 1) (+ level 1)))
     ((and (> level 0) (= (u8vector-ref bytes offset) 32)) level)
     (else 0))))

(def (token-if-nonempty kind start end)
  (if (< start end) (list (list 'token kind start end)) '()))

(def (skip-horizontal bytes offset end)
  (if (and (< offset end) (memv (u8vector-ref bytes offset) '(9 32)))
    (skip-horizontal bytes (+ offset 1) end)
    offset))

(def (scan-word bytes offset end)
  (if (and (< offset end)
           (not (memv (u8vector-ref bytes offset) '(9 32 10 13))))
    (scan-word bytes (+ offset 1) end)
    offset))

(def (strip-trailing-space bytes start end)
  (if (and (> end start)
           (memv (u8vector-ref bytes (- end 1)) '(9 10 13 32)))
    (strip-trailing-space bytes start (- end 1))
    end))

(def (ascii-lower-byte byte)
  (if (and (<= 65 byte) (<= byte 90)) (+ byte 32) byte))

(def (ascii-key-byte? byte)
  (or (and (<= 48 byte) (<= byte 57))
      (and (<= 65 byte) (<= byte 90))
      (and (<= 97 byte) (<= byte 122))
      (memv byte '(45 95))))

(def (scan-key bytes offset end)
  (if (and (< offset end) (ascii-key-byte? (u8vector-ref bytes offset)))
    (scan-key bytes (+ offset 1) end)
    offset))

(def (bytes-match? bytes start end marker case-insensitive?)
  (let* ((pattern (string->utf8 marker))
         (size (u8vector-length pattern)))
    (and (<= (+ start size) end)
         (let loop ((index 0))
           (or (= index size)
               (let ((actual (u8vector-ref bytes (+ start index)))
                     (expected (u8vector-ref pattern index)))
                 (and (= (if case-insensitive?
                           (ascii-lower-byte actual) actual)
                         (if case-insensitive?
                           (ascii-lower-byte expected) expected))
                      (loop (+ index 1)))))))))

(def (line-content-end bytes span)
  (let loop ((end (cdr span)))
    (if (and (> end (car span))
             (memv (u8vector-ref bytes (- end 1)) '(10 13)))
      (loop (- end 1))
      end)))

(def (line-marker? bytes span marker case-insensitive? indent?)
  (let* ((content-end (line-content-end bytes span))
         (start (if indent?
                  (skip-horizontal bytes (car span) content-end)
                  (car span)))
         (size (u8vector-length (string->utf8 marker))))
    (and (bytes-match? bytes start content-end marker case-insensitive?)
         (or (= (+ start size) content-end)
             (memv (u8vector-ref bytes (+ start size)) '(9 32))))))

(def (declared-key? rule bytes start end)
  (let (keys (key-line-keys rule))
    (or (null? keys)
        (ormap (lambda (key)
                 (and (= (- end start)
                         (u8vector-length (string->utf8 key)))
                      (bytes-match? bytes start end key
                                    (key-line-case-insensitive rule))))
               keys))))

(def (match-key-rule bytes span rule)
  (and (memq (key-line-node rule) '(OrgKeyword OrgBabelCall))
       (let* ((start (car span))
              (end (line-content-end bytes span))
              (indent (if (key-line-indent rule)
                        (skip-horizontal bytes start end) start))
              (prefix (key-line-prefix rule))
              (key-start (+ indent
                            (u8vector-length (string->utf8 prefix)))))
         (and (bytes-match? bytes indent end prefix
                            (key-line-case-insensitive rule))
              (let* ((key-end (scan-key bytes key-start end))
                     (separator (string->utf8 (key-line-separator rule))))
                (and (> key-end key-start)
                     (< key-end end)
                     (= (u8vector-ref bytes key-end)
                        (u8vector-ref separator 0))
                     (declared-key? rule bytes key-start key-end)
                     (list rule key-start key-end
                           (skip-horizontal bytes (+ key-end 1) end))))))))

(def (matching-key-line bytes span)
  (ormap (lambda (rule) (match-key-rule bytes span rule))
         (line-structure-key-lines org-v1-line-structure)))

(def (emit-key-line reversed bytes span match)
  (let* ((rule (car match))
         (key-start (cadr match))
         (key-end (caddr match))
         (value-start (cadddr match))
         (value-end (max value-start
                         (strip-trailing-space bytes value-start
                                               (cdr span))))
         (events
          (append
           (list (list 'start (key-line-node rule)))
           (token-if-nonempty (key-line-trivia-token rule)
                              (car span) key-start)
           (token-if-nonempty (key-line-key-token rule)
                              key-start key-end)
           (token-if-nonempty (key-line-trivia-token rule)
                              key-end value-start)
           (token-if-nonempty (key-line-value-token rule)
                              value-start value-end)
           (token-if-nonempty (key-line-trivia-token rule)
                              value-end (cdr span))
           (list '(finish)))))
    (foldl cons reversed events)))
