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
                 heading-fields-title-token heading-fields-trivia-token)
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
         (marker-bytes (string->utf8 marker))
         (size (u8vector-length marker-bytes)))
    (and (<= (+ start size) content-end)
         (let match ((index 0))
           (or (= index size)
               (and (let ((actual (u8vector-ref bytes (+ start index)))
                          (expected (u8vector-ref marker-bytes index)))
                      (= (if case-insensitive?
                           (ascii-lower-byte actual) actual)
                         (if case-insensitive?
                           (ascii-lower-byte expected) expected)))
                    (match (+ index 1)))))
         (or (= (+ start size) content-end)
             (memv (u8vector-ref bytes (+ start size)) '(9 32))))))

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

(def (emit-text-spans reversed spans)
  (foldl (lambda (span acc)
           (emit-line acc 'OrgTextLine 'TextLine (car span) (cdr span)))
         reversed (reverse spans)))

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
               (pending '()) (block #f))
      (cond
       ((null? rest)
        (reverse (cons '(finish)
                       (close-sections levels
                                       (if block
                                         (emit-text-spans reversed pending)
                                         reversed)))))
       (block
        (let* ((span (car rest))
               (level (org-headline-level bytes (car span) (cdr span))))
          (cond
           ((line-marker? bytes span (block-line-closing block)
                          (block-line-case-insensitive block)
                          (block-line-indent block))
            (loop (cdr rest) levels
                  (emit-source-block reversed bytes block pending span)
                  '() #f))
           ((and (block-line-heading-bound block) (> level 0))
            (loop rest levels (emit-text-spans reversed pending) '() #f))
           (else (loop (cdr rest) levels reversed
                       (cons span pending) block)))))
       (else
        (let* ((span (car rest))
               (start (car span))
               (end (cdr span))
               (level (org-headline-level bytes start end))
               (opening (and (= level 0)
                             (source-block-spec bytes span))))
          (cond
           ((> level 0)
            (let-values (((parents closed)
                          (close-through-level levels reversed level)))
              (loop (cdr rest) (cons level parents)
                    (emit-headline (cons '(start OrgSection) closed)
                                   bytes start end level)
                    '() #f)))
           (opening (loop (cdr rest) levels reversed (list span) opening))
           (else
            (loop (cdr rest) levels
                  (emit-line reversed 'OrgTextLine 'TextLine start end)
                  '() #f)))))))))
