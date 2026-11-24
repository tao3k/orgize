;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Parser-span-driven Org structural edit engine.
;;;
;;; Every rewrite resolves exact UTF-8 byte spans from `org-source-element-graph`,
;;; applies them through `:gerbil-parser/graph-support/edit`, and fails closed
;;; via full re-parse verification.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/type validate)
        (only-in :std/list/list filter find)
        (only-in :std/string/misc string-trim string-contains)
        (only-in :gerbil-parser/graph-support/edit
                 span-edit-insert span-edit-delete span-edit-replace
                 utf8-byte-slice apply-span-edits/verified)
        (only-in "../org-elements/source-graph.ss"
                 org-source-element-graph
                 org-source-element-graph/todo-profile
                 org-source-element-field)
        (only-in "../org-elements/query.ss"
                 make-org-element-query-context org-element-property)
        (only-in "../org-elements/view-helpers.ss"
                 org-graph-records-ordered org-find-headline org-headline-level
                 org-headline-tags org-headline-property-drawer
                 org-drawer-node-properties org-headline-planning-records
                 org-checkbox-state org-headline-checkbox-items)
        (only-in "types.ss" +org-edit-schema+ +org-edit-op-kind+
                 OrgEditOp org-edit-op? org-section-spec?)
        (only-in "emit.ss"
                 format-org-headline-line format-org-planning-line
                 format-org-property-line format-org-property-drawer
                 format-org-section format-org-babel-results))
(export +org-edit-default-todo-keywords+
        +org-edit-default-done-keywords+
        org-edit-set-todo
        org-edit-add-tag
        org-edit-remove-tag
        org-edit-set-planning
        org-edit-remove-planning
        org-edit-upsert-properties
        org-edit-delete-properties
        org-edit-set-checkbox
        org-edit-upsert-babel-results
        org-edit-upsert-section
        org-edit-append-section
        org-apply-edit
        org-apply-edits
        org-extract-subtree)

(def +org-edit-default-todo-keywords+
  '("TODO" "IN_PROGRESS" "NEXT" "STARTED" "BLOCKED" "HOLD" "WAITING" "REVIEW"))

(def +org-edit-default-done-keywords+
  '("DONE" "COMPLETED" "VERIFIED" "CANCELLED" "CANCELED" "ABORTED" "FAILED"))

(def (make-op op-val target-val payload-val)
  (validate OrgEditOp
            (.o bind-these: #f
                kind: +org-edit-op-kind+
                schema: +org-edit-schema+
                op: op-val
                target: target-val
                payload: payload-val)))

(def (org-edit-set-todo target new-keyword)
  (unless (or (not new-keyword)
              (and (string? new-keyword) (> (string-length (string-trim new-keyword)) 0)))
    (error "org-edit-set-todo requires #f or a non-empty keyword string" new-keyword))
  (make-op 'set-todo target (and new-keyword (string-trim new-keyword))))

(def (org-edit-add-tag target tag)
  (unless (and (string? tag) (> (string-length (string-trim tag)) 0))
    (error "org-edit-add-tag requires a non-empty tag string" tag))
  (make-op 'add-tag target (string-trim tag)))

(def (org-edit-remove-tag target tag)
  (unless (and (string? tag) (> (string-length (string-trim tag)) 0))
    (error "org-edit-remove-tag requires a non-empty tag string" tag))
  (make-op 'remove-tag target (string-trim tag)))

(def (org-edit-set-planning target key timestamp)
  (unless (and (string? key) (string? timestamp)
               (> (string-length (string-trim key)) 0)
               (> (string-length (string-trim timestamp)) 0))
    (error "org-edit-set-planning requires key and timestamp strings" key timestamp))
  (make-op 'set-planning target (cons (string-upcase (string-trim key))
                                      (string-trim timestamp))))

(def (org-edit-remove-planning target key)
  (unless (and (string? key) (> (string-length (string-trim key)) 0))
    (error "org-edit-remove-planning requires a planning key string" key))
  (make-op 'remove-planning target (string-upcase (string-trim key))))

(def (org-edit-upsert-properties target prop-pairs (preserve-existing? #f))
  (unless (and (list? prop-pairs)
               (andmap (lambda (p) (and (pair? p) (string? (car p)) (string? (cdr p))))
                       prop-pairs))
    (error "org-edit-upsert-properties requires an alist of (string . string)" prop-pairs))
  (make-op 'upsert-properties target (cons preserve-existing? prop-pairs)))

(def (org-edit-delete-properties target keys)
  (unless (and (list? keys) (andmap string? keys))
    (error "org-edit-delete-properties requires a list of key strings" keys))
  (make-op 'delete-properties target keys))

(def (org-edit-set-checkbox target item-index checked?)
  (unless (and (exact-integer? item-index) (>= item-index 0) (boolean? checked?))
    (error "org-edit-set-checkbox requires non-negative index and boolean" item-index checked?))
  (make-op 'set-checkbox target (cons item-index checked?)))

(def (org-edit-upsert-babel-results block-name body-text thinking-text: (thinking-text #f))
  (unless (and (string? block-name) (> (string-length (string-trim block-name)) 0)
               (string? body-text)
               (or (not thinking-text) (string? thinking-text)))
    (error "org-edit-upsert-babel-results requires valid block-name and body" block-name))
  (make-op 'upsert-babel-results (string-trim block-name) (cons body-text thinking-text)))

(def (org-edit-upsert-section target section-spec empty-preamble: (empty-preamble #f))
  (unless (org-section-spec? section-spec)
    (error "org-edit-upsert-section requires a validated OrgSectionSpec" section-spec))
  (make-op 'upsert-section target (cons section-spec empty-preamble)))

(def (org-edit-append-section section-spec empty-preamble: (empty-preamble #f))
  (unless (org-section-spec? section-spec)
    (error "org-edit-append-section requires a validated OrgSectionSpec" section-spec))
  (make-op 'append-section #f (cons section-spec empty-preamble)))

;;; ---------------------------------------------------------------------------
;;; Low-level UTF-8 byte boundary helpers (over parser-owned node spans)
;;; ---------------------------------------------------------------------------

(def (first-newline-in-span bytes start end)
  (let loop ((i start))
    (cond
     ((>= i end) end)
     ((= (u8vector-ref bytes i) 10) i)
     (else (loop (+ i 1))))))

(def (span-ends-with-newline? bytes start end)
  (and (> end start) (= (u8vector-ref bytes (- end 1)) 10)))

;; Inside a parser-owned OrgPropertyDrawer [drawer-start, drawer-end), the final
;; line is guaranteed by grammar to be the closing :END: line. Locate its exact
;; UTF-8 start offset by walking back from drawer-end over trailing newlines.
(def (drawer-closing-line-start bytes drawer-start drawer-end)
  (let loop-trim ((i drawer-end))
    (if (and (> i drawer-start)
             (let ((b (u8vector-ref bytes (- i 1))))
               (or (= b 10) (= b 13))))
      (loop-trim (- i 1))
      (let loop-bol ((j i))
        (cond
         ((<= j drawer-start) drawer-start)
         ((= (u8vector-ref bytes (- j 1)) 10) j)
         (else (loop-bol (- j 1))))))))

(def (headline-first-line-span bytes h)
  (let* ((s (.ref h 'start))
         (e (.ref h 'end))
         (nl (first-newline-in-span bytes s e)))
    (values s nl (if (< nl e) (+ nl 1) e))))

(def (require-target-headline ctx graph target op-name)
  (let ((h (org-find-headline ctx graph target)))
    (unless h
      (error (string-append "Org target headline not found for " op-name) target))
    h))

(def (parse-with-profile source todo-keywords done-keywords)
  (org-source-element-graph/todo-profile source todo-keywords done-keywords))

;;; ---------------------------------------------------------------------------
;;; Op -> List of Disjoint SpanEdits
;;; ---------------------------------------------------------------------------

(def (compile-headline-line-rewrite bytes ctx h next-todo next-tags label)
  (let-values (((s line-end _after-line) (headline-first-line-span bytes h)))
    (let* ((level (org-headline-level ctx h))
           (title (or (org-element-property ctx h "title") ""))
           (prio (org-element-property ctx h "priority"))
           (line (format-org-headline-line level title
                                           todo: next-todo
                                           priority: prio
                                           tags: next-tags)))
      (list (span-edit-replace s line-end line label: label)))))

(def (compile-set-planning bytes graph h key timestamp)
  (let* ((plannings (org-headline-planning-records graph h))
         (matched
          (find (lambda (p)
                  (let ((ks (org-source-element-field p "key")))
                    (and (list? ks)
                         (find (lambda (k) (and (string? k) (string-ci=? k key))) ks))))
                plannings))
         (line (format-org-planning-line key timestamp)))
    (if matched
      (let* ((ps (.ref matched 'start))
             (pe (.ref matched 'end))
             (rep (if (span-ends-with-newline? bytes ps pe)
                    (string-append line "\n")
                    line)))
        (list (span-edit-replace ps pe rep label: "set-planning")))
      (let-values (((_s line-end after-line) (headline-first-line-span bytes h)))
        (if (< line-end after-line)
          (list (span-edit-insert after-line (string-append line "\n") label: "insert-planning"))
          (list (span-edit-insert line-end (string-append "\n" line "\n") label: "insert-planning")))))))

(def (compile-remove-planning graph h key)
  (let* ((plannings (org-headline-planning-records graph h))
         (matched
          (filter (lambda (p)
                    (let ((ks (org-source-element-field p "key")))
                      (and (list? ks)
                           (find (lambda (k) (and (string? k) (string-ci=? k key))) ks))))
                  plannings)))
    (map (lambda (p)
           (span-edit-delete (.ref p 'start) (.ref p 'end) label: "remove-planning"))
         matched)))

(def (compile-upsert-properties bytes graph h preserve-existing? prop-pairs)
  (if (null? prop-pairs)
    '()
    (let ((drawer (org-headline-property-drawer graph h)))
      (if (not drawer)
        (let* ((plannings (org-headline-planning-records graph h))
               (insert-pos
                (if (pair? plannings)
                  (.ref (car (reverse plannings)) 'end)
                  (let-values (((_s _le after-line) (headline-first-line-span bytes h)))
                    after-line)))
               (needs-leading-nl?
                (and (> insert-pos 0)
                     (not (= (u8vector-ref bytes (- insert-pos 1)) 10))))
               (text (string-append (if needs-leading-nl? "\n" "")
                                    (format-org-property-drawer prop-pairs))))
          (list (span-edit-insert insert-pos text label: "insert-property-drawer")))
        (let* ((existing (org-drawer-node-properties graph drawer))
               (edits '())
               (to-append '()))
          (for-each
           (lambda (pair)
             (let* ((k (car pair))
                    (v (cdr pair))
                    (found (find (lambda (p)
                                   (let ((pk (org-source-element-field p "key")))
                                     (and (string? pk) (string-ci=? pk k))))
                                 existing)))
               (cond
                ((and found preserve-existing?) (void))
                (found
                 (let* ((ps (.ref found 'start))
                        (pe (.ref found 'end))
                        (line (format-org-property-line k v))
                        (rep (if (span-ends-with-newline? bytes ps pe)
                               (string-append line "\n")
                               line)))
                   (set! edits (cons (span-edit-replace ps pe rep label: (string-append "prop:" k))
                                     edits))))
                (else
                 (set! to-append (cons pair to-append))))))
           prop-pairs)
          (let ((insert-edits
                 (if (null? to-append)
                   '()
                   (let* ((end-bol (drawer-closing-line-start bytes (.ref drawer 'start) (.ref drawer 'end)))
                          (lines (apply string-append
                                        (map (lambda (p)
                                               (string-append (format-org-property-line (car p) (cdr p)) "\n"))
                                             (reverse to-append)))))
                     (list (span-edit-insert end-bol lines label: "append-properties"))))))
            (append edits insert-edits)))))))

(def (compile-delete-properties graph h keys)
  (let ((drawer (org-headline-property-drawer graph h)))
    (if (not drawer)
      '()
      (let* ((existing (org-drawer-node-properties graph drawer))
             (matched
              (filter (lambda (p)
                        (let ((pk (org-source-element-field p "key")))
                          (and (string? pk)
                               (find (lambda (k) (string-ci=? pk k)) keys))))
                      existing)))
        (map (lambda (p)
               (span-edit-delete (.ref p 'start) (.ref p 'end) label: "delete-property"))
             matched)))))

(def (compile-set-checkbox bytes ctx graph h item-index checked?)
  (let ((boxes (org-headline-checkbox-items ctx graph h #t)))
    (unless (< item-index (length boxes))
      (error "Checklist item index out of range" item-index (length boxes)))
    (let* ((item-rec (list-ref boxes item-index))
           (is (.ref item-rec 'start))
           (ie (.ref item-rec 'end))
           (cb-offset
            (let loop ((i is))
              (if (> (+ i 3) ie)
                (error "Malformed checkbox token inside OrgListItem span" is ie)
                (if (and (= (u8vector-ref bytes i) 91)       ;; '['
                         (= (u8vector-ref bytes (+ i 2)) 93)) ;; ']'
                  i
                  (loop (+ i 1))))))
           (cb-edit (span-edit-replace cb-offset (+ cb-offset 3)
                                       (if checked? "[X]" "[ ]")
                                       label: "set-checkbox"))
           (done-count
            (let loop ((rest boxes) (idx 0) (acc 0))
              (if (null? rest)
                acc
                (let* ((st (org-checkbox-state (org-element-property ctx (car rest) "checkbox")))
                       (done? (if (= idx item-index) checked? (eq? st 'checked))))
                  (loop (cdr rest) (+ idx 1) (if done? (+ acc 1) acc))))))
           (total-count (length boxes))
           (cookie
            (find (lambda (r)
                    (and (equal? (.ref r 'kind) "statistics-cookie")
                         (equal? (.ref r 'parent) (.ref h 'id))))
                  (org-graph-records-ordered graph)))
           (cookie-edits
            (if (not cookie)
              '()
              (let* ((cval (or (org-source-element-field cookie "value") ""))
                     (pct? (and (string? cval) (string-contains cval "%")))
                     (new-cookie
                      (if pct?
                        (string-append "[" (number->string (quotient (* done-count 100) total-count)) "%]")
                        (string-append "[" (number->string done-count) "/" (number->string total-count) "]"))))
                (list (span-edit-replace (.ref cookie 'start) (.ref cookie 'end)
                                         new-cookie label: "update-statistics-cookie"))))))
      (cons cb-edit cookie-edits))))

(def (compile-upsert-babel-results bytes graph target-name body-text thinking-text)
  (let* ((records (org-graph-records-ordered graph))
         (keywords (filter (lambda (r) (equal? (.ref r 'kind) "keyword")) records))
         (drawers (filter (lambda (r) (equal? (.ref r 'kind) "drawer")) records))
         (src-blocks (filter (lambda (r) (equal? (.ref r 'kind) "src-block")) records))
         (matched-block
          (find (lambda (sb)
                  (find (lambda (kw)
                          (and (equal? (.ref kw 'parent) (.ref sb 'parent))
                               (string-ci=? (or (org-source-element-field kw "key") "") "NAME")
                               (equal? (string-trim (or (org-source-element-field kw "value") ""))
                                       target-name)
                               (<= (.ref kw 'end) (.ref sb 'start))
                               (equal? (string-trim (utf8-byte-slice bytes (.ref kw 'end) (.ref sb 'start))) "")))
                        keywords))
                src-blocks)))
    (unless matched-block
      (error "Named Org-Babel source block not found" target-name))
    (let* ((res-kw
            (find (lambda (kw)
                    (and (equal? (.ref kw 'parent) (.ref matched-block 'parent))
                         (string-ci=? (or (org-source-element-field kw "key") "") "RESULTS")
                         (equal? (string-trim (or (org-source-element-field kw "value") ""))
                                 target-name)
                         (>= (.ref kw 'start) (.ref matched-block 'end))))
                  keywords))
           (res-drawer
            (and res-kw
                 (find (lambda (dr)
                         (and (equal? (.ref dr 'parent) (.ref matched-block 'parent))
                              (string-ci=? (or (org-source-element-field dr "name") "") "RESULTS")
                              (>= (.ref dr 'start) (.ref res-kw 'end))
                              (equal? (string-trim (utf8-byte-slice bytes (.ref res-kw 'end) (.ref dr 'start))) "")))
                       drawers)))
           (formatted (format-org-babel-results target-name body-text thinking-text)))
      (if (and res-kw res-drawer)
        (list (span-edit-replace (.ref res-kw 'start) (.ref res-drawer 'end)
                                 formatted label: "replace-babel-results"))
        (let* ((insert-pos (.ref matched-block 'end))
               (sep (if (and (> insert-pos 0) (= (u8vector-ref bytes (- insert-pos 1)) 10))
                      "\n" "\n\n")))
          (list (span-edit-insert insert-pos (string-append sep formatted)
                                  label: "insert-babel-results")))))))

(def (compile-append-section bytes source-text section-spec empty-preamble)
  (let* ((total (u8vector-length bytes))
         (formatted (format-org-section section-spec))
         (blank? (equal? (string-trim source-text) "")))
    (if blank?
      (list (span-edit-replace 0 total
                               (string-append (or empty-preamble "") formatted)
                               label: "init-section"))
      (let* ((trim-end
              (let loop ((i total))
                (if (and (> i 0)
                         (let ((b (u8vector-ref bytes (- i 1))))
                           (or (= b 10) (= b 13) (= b 32) (= b 9))))
                  (loop (- i 1))
                  i))))
        (list (span-edit-replace trim-end total
                                 (string-append "\n\n" formatted)
                                 label: "append-section"))))))

(def (compile-upsert-section bytes source-text ctx graph target section-spec empty-preamble)
  (let ((existing (and (not (equal? (string-trim source-text) ""))
                       (org-find-headline ctx graph target))))
    (if existing
      (list (span-edit-replace (.ref existing 'start) (.ref existing 'end)
                               (format-org-section section-spec)
                               label: "replace-section"))
      (compile-append-section bytes source-text section-spec empty-preamble))))

;;; ---------------------------------------------------------------------------
;;; Public Verified Application Surface
;;; ---------------------------------------------------------------------------

(def (org-apply-edit source-text op
                     todo-keywords: (todo-keywords +org-edit-default-todo-keywords+)
                     done-keywords: (done-keywords +org-edit-default-done-keywords+))
  (unless (string? source-text)
    (error "org-apply-edit requires an Org source string" source-text))
  (unless (org-edit-op? op)
    (error "org-apply-edit requires a validated OrgEditOp" op))
  (let* ((bytes (string->utf8 source-text))
         (blank? (equal? (string-trim source-text) ""))
         (graph (if blank? #f (parse-with-profile source-text todo-keywords done-keywords)))
         (ctx (and graph (make-org-element-query-context graph)))
         (kind (.ref op 'op))
         (target (.ref op 'target))
         (payload (.ref op 'payload))
         (edits
          (case kind
            ((set-todo)
             (let ((h (require-target-headline ctx graph target "set-todo")))
               (compile-headline-line-rewrite bytes ctx h payload (org-headline-tags ctx h) "set-todo")))
            ((add-tag)
             (let* ((h (require-target-headline ctx graph target "add-tag"))
                    (cur (org-headline-tags ctx h))
                    (has? (find (lambda (t) (string-ci=? t payload)) cur))
                    (next (if has? cur (append cur (list payload)))))
               (if has?
                 '()
                 (compile-headline-line-rewrite bytes ctx h
                                                (org-element-property ctx h "todo-keyword")
                                                next "add-tag"))))
            ((remove-tag)
             (let* ((h (require-target-headline ctx graph target "remove-tag"))
                    (cur (org-headline-tags ctx h))
                    (next (filter (lambda (t) (not (string-ci=? t payload))) cur)))
               (if (= (length next) (length cur))
                 '()
                 (compile-headline-line-rewrite bytes ctx h
                                                (org-element-property ctx h "todo-keyword")
                                                next "remove-tag"))))
            ((set-planning)
             (let ((h (require-target-headline ctx graph target "set-planning")))
               (compile-set-planning bytes graph h (car payload) (cdr payload))))
            ((remove-planning)
             (let ((h (require-target-headline ctx graph target "remove-planning")))
               (compile-remove-planning graph h payload)))
            ((upsert-properties)
             (let ((h (require-target-headline ctx graph target "upsert-properties")))
               (compile-upsert-properties bytes graph h (car payload) (cdr payload))))
            ((delete-properties)
             (let ((h (require-target-headline ctx graph target "delete-properties")))
               (compile-delete-properties graph h payload)))
            ((set-checkbox)
             (let ((h (require-target-headline ctx graph target "set-checkbox")))
               (compile-set-checkbox bytes ctx graph h (car payload) (cdr payload))))
            ((upsert-babel-results)
             (when blank?
               (error "Named Org-Babel source block not found in empty document" target))
             (compile-upsert-babel-results bytes graph target (car payload) (cdr payload)))
            ((upsert-section)
             (compile-upsert-section bytes source-text ctx graph target (car payload) (cdr payload)))
            ((append-section)
             (compile-append-section bytes source-text (car payload) (cdr payload)))
            (else (error "unsupported OrgEditOp" kind)))))
    (if (null? edits)
      source-text
      (.ref (apply-span-edits/verified
             source-text edits
             (lambda (next-src) (parse-with-profile next-src todo-keywords done-keywords)))
            'source))))

(def (org-apply-edits source-text ops
                      todo-keywords: (todo-keywords +org-edit-default-todo-keywords+)
                      done-keywords: (done-keywords +org-edit-default-done-keywords+))
  (unless (list? ops)
    (error "org-apply-edits requires a list of OrgEditOp values" ops))
  (let loop ((rest ops) (current source-text))
    (if (null? rest)
      current
      (loop (cdr rest)
            (org-apply-edit current (car rest)
                            todo-keywords: todo-keywords
                            done-keywords: done-keywords)))))

(def (org-extract-subtree source-text target
                          extra-properties: (extra-properties '())
                          preserve-existing?: (preserve-existing? #t)
                          guard-fn: (guard-fn #f)
                          todo-keywords: (todo-keywords +org-edit-default-todo-keywords+)
                          done-keywords: (done-keywords +org-edit-default-done-keywords+))
  (unless (string? source-text)
    (error "org-extract-subtree requires an Org source string" source-text))
  (let* ((bytes (string->utf8 source-text))
         (graph (parse-with-profile source-text todo-keywords done-keywords))
         (ctx (make-org-element-query-context graph))
         (h (require-target-headline ctx graph target "extract-subtree")))
    (when guard-fn
      (guard-fn ctx graph h))
    (let* ((raw-subtree (utf8-byte-slice bytes (.ref h 'start) (.ref h 'end)))
           (promoted
            (if (null? extra-properties)
              raw-subtree
              (org-apply-edit raw-subtree
                              (org-edit-upsert-properties 'first extra-properties preserve-existing?)
                              todo-keywords: todo-keywords
                              done-keywords: done-keywords)))
           (remaining
            (.ref (apply-span-edits/verified
                   source-text
                   (list (span-edit-delete (.ref h 'start) (.ref h 'end) label: "extract-subtree"))
                   (lambda (next-src)
                     (if (equal? (string-trim next-src) "")
                       #t
                       (parse-with-profile next-src todo-keywords done-keywords))))
                  'source)))
      (values remaining promoted))))
