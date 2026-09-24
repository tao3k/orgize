;;; -*- Gerbil -*-
;;; Org-owned section containment and opaque block recovery.
;;; This pass is not a runtime dependency of Cargo consumers.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-blocks block-line-block-node
                 block-line-opening block-line-closing
                 block-line-case-insensitive block-line-indent
                 block-line-heading-bound block-line-begin-token
                 block-line-body-token block-line-end-token
                 block-line-contents block-line-body-line
                 block-line-header block-header-argument-token
                 block-header-trivia-token line-structure-heading
                 heading-line-heading-token heading-line-fields
                 heading-fields-title-token heading-fields-trivia-token)
        (only-in "modules/org-parser/funs.ss"
                 org-line-spans structural-lead-byte
                 org-headline-level token-if-nonempty
                 skip-horizontal scan-word strip-trailing-space
                 line-marker? matching-key-line emit-key-line)
        (only-in "modules/org-parser/table.ss"
                 org-table-line? org-table-node emit-org-table-row)
        (only-in "modules/org-parser/property.ss"
                 parse-property-drawer emit-property-drawer)
        (only-in "modules/org-parser/link.ss" emit-org-text-line)
        (only-in "parser.ss" org-v1-line-structure))
(export parse-org-outline-events)

(def (close-sections levels reversed)
  (if (null? levels)
    reversed
    (close-sections (cdr levels) (cons '(finish) reversed))))

(def (close-through-level levels reversed level)
  (if (or (null? levels) (< (car levels) level))
    (values levels reversed)
    (close-through-level (cdr levels) (cons '(finish) reversed) level)))

(def (close-paragraph reversed open?)
  (if open? (cons '(finish) reversed) reversed))

(def (blank-line? bytes span)
  (let loop ((offset (car span)))
    (or (= offset (cdr span))
        (and (memv (u8vector-ref bytes offset) '(9 10 13 32))
             (loop (+ offset 1))))))

(def (emit-paragraph-line reversed open? bytes span)
  (if (blank-line? bytes span)
    (values (emit-org-text-line (close-paragraph reversed open?)
                                bytes (car span) (cdr span))
            #f)
    (values (emit-org-text-line
             (if open? reversed (cons '(start OrgParagraph) reversed))
             bytes (car span) (cdr span))
            #t)))

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

(def (closed-block-spec bytes span)
  (ormap (lambda (block)
           (and (eq? (block-line-contents block) 'opaque)
                (line-marker? bytes span (block-line-opening block)
                              (block-line-case-insensitive block)
                              (block-line-indent block))
                block))
         (line-structure-blocks org-v1-line-structure)))

(def (emit-text-spans reversed bytes spans)
  (foldl (lambda (span state)
           (let-values (((events open?)
                         (emit-paragraph-line (car state) (cdr state)
                                              bytes span)))
             (cons events open?)))
         (cons reversed #f) (reverse spans)))

(def (emit-opaque-block reversed bytes block pending closing)
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
         (argument-start (and header
                              (skip-horizontal bytes prefix-end
                                               (cdr opening))))
         (argument-end (and header
                            (scan-word bytes argument-start
                                       (cdr opening))))
         (events
          (append
           (list (list 'start (block-line-block-node block))
                 (list 'token (block-line-begin-token block)
                       (car opening)
                       (if header prefix-end (cdr opening))))
           (if header
             (append
              (token-if-nonempty (block-header-trivia-token header)
                                 prefix-end argument-start)
              (token-if-nonempty (block-header-argument-token header)
                                 argument-start argument-end)
              (token-if-nonempty (block-header-trivia-token header)
                                 argument-end (cdr opening)))
             '())
           (map (lambda (span) (list 'token (block-line-body-token block)
                                     (car span) (cdr span))) body)
           (list (list 'token (block-line-end-token block)
                       (car closing) (cdr closing))
                 '(finish)))))
    (foldl cons reversed events)))

(def (parse-org-outline-events source)
  (let* ((bytes (string->utf8 source))
         (spans (org-line-spans bytes)))
    (let loop ((rest spans) (levels '())
               (reversed '((start OrgFile)))
               (pending '()) (block #f) (paragraph? #f)
               (table? #f) (after-heading? #f))
      (cond
       ((null? rest)
        (let* ((state (if block
                        (emit-text-spans reversed bytes pending)
                        (cons reversed paragraph?)))
               (closed (close-paragraph (car state) (cdr state)))
               (closed (if table? (cons '(finish) closed) closed)))
          (reverse (cons '(finish) (close-sections levels closed)))))
       (block
        (let* ((span (car rest))
               (level (org-headline-level bytes (car span) (cdr span))))
          (cond
           ((line-marker? bytes span (block-line-closing block)
                          (block-line-case-insensitive block)
                          (block-line-indent block))
            (if (block-line-body-line block)
              (let (parsed (parse-property-drawer bytes pending block))
                (if parsed
                (loop (cdr rest) levels
                      (emit-property-drawer reversed block parsed span)
                      '() #f #f #f #f)
                (let (state (emit-text-spans reversed bytes
                                            (cons span pending)))
                  (loop (cdr rest) levels (car state)
                        '() #f (cdr state) #f #f))))
              (loop (cdr rest) levels
                    (emit-opaque-block reversed bytes block pending span)
                    '() #f #f #f #f)))
           ((and (block-line-heading-bound block) (> level 0))
            (let (state (emit-text-spans reversed bytes pending))
              (loop rest levels (car state) '() #f (cdr state) #f #f)))
           (else (loop (cdr rest) levels reversed
                       (cons span pending) block #f #f #f)))))
       ((and table? (not (org-table-line? bytes (car rest))))
        (loop rest levels (cons '(finish) reversed) '() #f #f #f #f))
       (else
        (let* ((span (car rest))
               (start (car span))
               (end (cdr span))
               (lead (structural-lead-byte bytes span))
               (table-line? (org-table-line? bytes span))
               (level (org-headline-level bytes start end))
               (opening (and (not table-line?) (= level 0)
                             (memv lead '(35 58))
                             (closed-block-spec bytes span)))
               (key-line (and (not table-line?) (= level 0) (not opening)
                              (memv lead '(35 83 115 68 100 67 99))
                              (matching-key-line bytes span
                                                 after-heading?))))
          (cond
           (table-line?
            (loop (cdr rest) levels
                  (emit-org-table-row
                   (if table? reversed
                       (cons (list 'start org-table-node)
                             (close-paragraph reversed paragraph?)))
                   bytes span)
                  '() #f #f #t #f))
           ((> level 0)
            (let-values (((parents closed)
                          (close-through-level
                           levels (close-paragraph reversed paragraph?)
                           level)))
              (loop (cdr rest) (cons level parents)
                    (emit-headline (cons '(start OrgSection) closed)
                                   bytes start end level)
                    '() #f #f #f #t)))
           (opening
            (loop (cdr rest) levels
                  (close-paragraph reversed paragraph?)
                  (list span) opening #f #f #f))
           (key-line
            (loop (cdr rest) levels
                  (emit-key-line (close-paragraph reversed paragraph?)
                                 bytes span key-line)
                  '() #f #f #f #f))
           (else
            (let-values (((events open?)
                          (emit-paragraph-line reversed paragraph?
                                               bytes span)))
              (loop (cdr rest) levels events '() #f open? #f #f))))))))))
