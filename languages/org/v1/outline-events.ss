;;; -*- Gerbil -*-
;;; Org-owned section containment and opaque source-block recovery.
;;; This pass is not a runtime dependency of Cargo consumers.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-blocks block-line-block-node
                 block-line-opening block-line-closing
                 block-line-case-insensitive block-line-indent
                 block-line-heading-bound block-line-begin-token
                 block-line-body-token block-line-end-token
                 block-line-header block-header-argument-token
                 block-header-trivia-token line-structure-heading
                 heading-line-heading-token heading-line-fields
                 heading-fields-title-token heading-fields-trivia-token
                 line-structure-key-lines key-line-prefix key-line-keys
                 key-line-separator key-line-case-insensitive
                 key-line-indent key-line-node key-line-key-token
                 key-line-value-token key-line-trivia-token)
        (only-in "line-event-parser.ss" parse-org-line-events)
        (only-in "parser.ss" org-v1-line-structure))
(export parse-org-outline-events org-headline-level)

(def (org-headline-level bytes start end)
  (let loop ((offset start) (level 0))
    (cond
     ((>= offset end) 0)
     ((= (u8vector-ref bytes offset) 42)
      (loop (+ offset 1) (+ level 1)))
     ((and (> level 0) (= (u8vector-ref bytes offset) 32)) level)
     (else 0))))

(def (close-sections levels reversed)
  (if (null? levels)
    reversed
    (close-sections (cdr levels) (cons '(finish) reversed))))

(def (close-through-level levels reversed level)
  (if (or (null? levels) (< (car levels) level))
    (values levels reversed)
    (close-through-level (cdr levels) (cons '(finish) reversed) level)))

(def (emit-line reversed node token start end)
  (cons '(finish)
        (cons (list 'token token start end)
              (cons (list 'start node) reversed))))

(def (close-paragraph reversed open?)
  (if open? (cons '(finish) reversed) reversed))

(def (blank-line? bytes span)
  (let loop ((offset (car span)))
    (or (= offset (cdr span))
        (and (memv (u8vector-ref bytes offset) '(9 10 13 32))
             (loop (+ offset 1))))))

(def (emit-paragraph-line reversed open? bytes span)
  (if (blank-line? bytes span)
    (values (emit-line (close-paragraph reversed open?)
                       'OrgTextLine 'TextLine (car span) (cdr span))
            #f)
    (values (emit-line (if open? reversed
                        (cons '(start OrgParagraph) reversed))
                       'OrgTextLine 'TextLine (car span) (cdr span))
            #t)))

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

(def (emit-headline reversed bytes start end level)
  (let* ((heading (line-structure-heading org-v1-line-structure))
         (fields (heading-line-fields heading))
         (title-start (skip-horizontal bytes (+ start level) end))
         (title-end (max title-start
                         (strip-trailing-space bytes title-start end)))
         (events
          (append
           (list '(start OrgHeadline)
                 (list 'token (heading-line-heading-token heading)
                       start (+ start level)))
           (token-if-nonempty (heading-fields-trivia-token fields)
                              (+ start level) title-start)
           (token-if-nonempty (heading-fields-title-token fields)
                              title-start title-end)
           (token-if-nonempty (heading-fields-trivia-token fields)
                              title-end end)
           (list '(finish)))))
    (foldl cons reversed events)))

(def (line-spans flat)
  (let loop ((rest (cdr flat)) (reversed '()))
    (if (equal? (car rest) '(finish))
      (reverse reversed)
      (let (token (cadr rest))
        (loop (cdddr rest) (cons (cons (caddr token) (cadddr token))
                                reversed))))))

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
         (start (let skip ((offset (car span)))
                  (if (and indent? (< offset content-end)
                           (memv (u8vector-ref bytes offset) '(9 32)))
                    (skip (+ offset 1))
                    offset)))
         (size (u8vector-length (string->utf8 marker))))
    (and (bytes-match? bytes start content-end marker case-insensitive?)
         (or (= (+ start size) content-end)
             (memv (u8vector-ref bytes (+ start size)) '(9 32))))))

(def (matching-key-line bytes span)
  (let ((start (car span)) (end (line-content-end bytes span)))
    (let loop ((rules (line-structure-key-lines org-v1-line-structure)))
      (and (pair? rules)
           (let* ((rule (car rules))
                  (prefix (key-line-prefix rule))
                  (indent (if (key-line-indent rule)
                            (skip-horizontal bytes start end) start))
                  (key-start (+ indent
                                (u8vector-length (string->utf8 prefix)))))
             (if (and (memq (key-line-node rule)
                            '(OrgKeyword OrgBabelCall))
                      (bytes-match? bytes indent end prefix
                                    (key-line-case-insensitive rule)))
               (let* ((key-end (scan-key bytes key-start end))
                      (separator (string->utf8 (key-line-separator rule)))
                      (keys (key-line-keys rule)))
                 (if (and (> key-end key-start)
                          (< key-end end)
                          (= (u8vector-ref bytes key-end)
                             (u8vector-ref separator 0))
                          (or (null? keys)
                              (ormap (lambda (key)
                                       (and (= (- key-end key-start)
                                               (u8vector-length (string->utf8 key)))
                                            (bytes-match? bytes key-start key-end key
                                                          (key-line-case-insensitive rule))))
                                     keys)))
                   (list rule key-start key-end
                         (skip-horizontal bytes (+ key-end 1) end))
                   (loop (cdr rules))))
               (loop (cdr rules))))))))

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

(def (source-block-spec bytes span)
  (let loop ((blocks (line-structure-blocks org-v1-line-structure)))
    (and (pair? blocks)
         (let (block (car blocks))
           (if (and (eq? (block-line-block-node block) 'OrgSourceBlock)
                    (line-marker? bytes span (block-line-opening block)
                                  (block-line-case-insensitive block)
                                  (block-line-indent block)))
             block
             (loop (cdr blocks)))))))

(def (emit-text-spans reversed bytes spans)
  (foldl (lambda (span state)
           (let-values (((events open?)
                         (emit-paragraph-line (car state) (cdr state)
                                              bytes span)))
             (cons events open?)))
         (cons reversed #f) (reverse spans)))

(def (emit-source-block reversed bytes block pending closing)
  (let* ((lines (reverse pending))
         (opening (car lines))
         (body (cdr lines))
         (header (block-line-header block))
         (prefix-end (let* ((indent-end
                            (if (block-line-indent block)
                              (skip-horizontal bytes (car opening)
                                               (cdr opening))
                              (car opening))))
                       (+ indent-end
                          (u8vector-length
                           (string->utf8 (block-line-opening block))))))
         (argument-start (skip-horizontal bytes prefix-end (cdr opening)))
         (argument-end (scan-word bytes argument-start (cdr opening)))
         (events
          (append
           (list '(start OrgSourceBlock)
                 (list 'token (block-line-begin-token block)
                       (car opening) prefix-end))
           (token-if-nonempty (block-header-trivia-token header)
                              prefix-end argument-start)
           (token-if-nonempty (block-header-argument-token header)
                              argument-start argument-end)
           (token-if-nonempty (block-header-trivia-token header)
                              argument-end (cdr opening))
           (map (lambda (span) (list 'token (block-line-body-token block)
                                     (car span) (cdr span))) body)
           (list (list 'token (block-line-end-token block)
                       (car closing) (cdr closing))
                 '(finish)))))
    (foldl cons reversed events)))

(def (parse-org-outline-events source)
  (let* ((bytes (string->utf8 source))
         (spans (line-spans (parse-org-line-events source))))
    (let loop ((rest spans) (levels '())
               (reversed '((start OrgFile)))
               (pending '()) (block #f) (paragraph? #f))
      (cond
       ((null? rest)
        (let* ((state (if block
                        (emit-text-spans reversed bytes pending)
                        (cons reversed paragraph?)))
               (closed (close-paragraph (car state) (cdr state))))
          (reverse (cons '(finish) (close-sections levels closed)))))
       (block
        (let* ((span (car rest))
               (level (org-headline-level bytes (car span) (cdr span))))
          (cond
           ((line-marker? bytes span (block-line-closing block)
                          (block-line-case-insensitive block)
                          (block-line-indent block))
            (loop (cdr rest) levels
                  (emit-source-block reversed bytes block pending span)
                  '() #f #f))
           ((and (block-line-heading-bound block) (> level 0))
            (let (state (emit-text-spans reversed bytes pending))
              (loop rest levels (car state) '() #f (cdr state))))
           (else (loop (cdr rest) levels reversed
                       (cons span pending) block #f)))))
       (else
        (let* ((span (car rest))
               (start (car span))
               (end (cdr span))
               (level (org-headline-level bytes start end))
               (opening (and (= level 0)
                             (source-block-spec bytes span)))
               (key-line (and (= level 0) (not opening)
                              (matching-key-line bytes span))))
          (cond
           ((> level 0)
            (let-values (((parents closed)
                          (close-through-level
                           levels (close-paragraph reversed paragraph?)
                           level)))
              (loop (cdr rest) (cons level parents)
                    (emit-headline (cons '(start OrgSection) closed)
                                   bytes start end level)
                    '() #f #f)))
           (opening
            (loop (cdr rest) levels
                  (close-paragraph reversed paragraph?)
                  (list span) opening #f))
           (key-line
            (loop (cdr rest) levels
                  (emit-key-line (close-paragraph reversed paragraph?)
                                 bytes span key-line)
                  '() #f #f))
           (else
            (let-values (((events open?)
                          (emit-paragraph-line reversed paragraph?
                                               bytes span)))
              (loop (cdr rest) levels events '() #f open?))))))))))
