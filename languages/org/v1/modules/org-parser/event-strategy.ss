;;; -*- Gerbil -*-
;;; Org-owned, compositional event strategy. POO parser declarations own markers;
;;; the resulting forms execute in Scheme and AOT-lower to one Rust function.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-heading line-structure-blocks
                 line-structure-key-lines key-line-node key-line-prefix
                 key-line-keys key-line-separator key-line-key-token
                 key-line-value-token key-line-trivia-token
                 heading-line-marker heading-line-separator
                 block-line-block-node block-line-opening block-line-closing
                 block-line-body-line block-line-begin-token
                 block-line-body-token block-line-end-token block-line-header
                 block-header-argument-token block-header-trivia-token
                 key-value-line-marker line-structure-table
                 table-line-delimiter table-line-table-node
                 table-line-row-node table-line-rule-row-node
                 table-line-cell-node table-line-separator-token
                 table-line-cell-token table-line-trivia-token
                 table-line-rule-token)
        (only-in "../../parser.ss" org-v1-line-structure))
(export org-event-initial org-event-line-forms org-event-finish-forms)

(def (block-by-node node)
  (or (ormap (lambda (block)
               (and (eq? (block-line-block-node block) node) block))
             (line-structure-blocks org-v1-line-structure))
      (error "missing Org block declaration" node)))

(def (key-line-by-node node)
  (or (ormap (lambda (rule) (and (eq? (key-line-node rule) node) rule))
             (line-structure-key-lines org-v1-line-structure))
      (error "missing Org key-line declaration" node)))

(def property-rule (block-by-node 'OrgPropertyDrawer))
(def heading-rule (line-structure-heading org-v1-line-structure))
(def keyword-rule (key-line-by-node 'OrgKeyword))
(def babel-call-rule (key-line-by-node 'OrgBabelCall))
(def planning-rule (key-line-by-node 'OrgPlanning))
(def clock-rule (key-line-by-node 'OrgClock))
(def table-rule (line-structure-table org-v1-line-structure))
(def table-byte
  (char->integer (string-ref (table-line-delimiter table-rule) 0)))
(def property-open (block-line-opening property-rule))
(def property-close (block-line-closing property-rule))
(def property-marker (key-value-line-marker (block-line-body-line property-rule)))
(def heading-marker (heading-line-marker heading-rule))
(def heading-separator (heading-line-separator heading-rule))
(def keyword-prefix (key-line-prefix keyword-rule))
(def babel-call-marker
  (let (keys (key-line-keys babel-call-rule))
    (unless (and (pair? keys) (null? (cdr keys)))
      (error "Org Babel CALL requires one declared key" keys))
    (string-append (key-line-prefix babel-call-rule)
                   (car keys) (key-line-separator babel-call-rule))))

(def (numbered-blocks names)
  (let loop ((rest names) (id 1))
    (if (null? rest) '()
      (cons (cons id (block-by-node (car rest)))
            (loop (cdr rest) (+ id 1))))))

(def opaque-blocks
  (numbered-blocks
   '(OrgSourceBlock OrgExampleBlock OrgCommentBlock OrgExportBlock)))

(def close-paragraph
  '(if (state paragraph-open)
       ((finish-node) (set-bool paragraph-open (bool #f))) ()))

(def (paragraph-form)
  `(if (line-blank?)
       (,close-paragraph
        (start-node OrgTextLine) (token TextLine start end) (finish-node))
       ((if (not (state paragraph-open))
            ((start-node OrgParagraph) (set-bool paragraph-open (bool #t))) ())
        (start-node OrgTextLine) (token TextLine start end) (finish-node))))

(def (keyword-form)
  `(if (line-has-key-after-prefix? ,keyword-prefix)
       (,close-paragraph
        (if (line-starts-with-ascii-ci ,babel-call-marker)
            ((start-node OrgBabelCall)) ((start-node OrgKeyword)))
        (token KeywordTrivia start (line-prefix-end ,keyword-prefix))
        (token KeywordKey (line-prefix-end ,keyword-prefix)
               (line-scan-key (line-prefix-end ,keyword-prefix)))
        (token KeywordTrivia
               (line-scan-key (line-prefix-end ,keyword-prefix))
               (line-skip-horizontal
                (line-step (line-scan-key (line-prefix-end ,keyword-prefix)))))
        (token KeywordValue
               (line-skip-horizontal
                (line-step (line-scan-key (line-prefix-end ,keyword-prefix))))
               (line-trim-end-from
                (line-skip-horizontal
                 (line-step (line-scan-key (line-prefix-end ,keyword-prefix))))))
        (token KeywordTrivia
               (line-trim-end-from
                (line-skip-horizontal
                 (line-step (line-scan-key (line-prefix-end ,keyword-prefix))))) end)
        (finish-node))
       (,(paragraph-form))))

(def (declared-key-form rule key otherwise)
  (let* ((marker (string-append (key-line-prefix rule) key
                                (key-line-separator rule)))
         (key-end `(line-prefix-end ,key))
         (value-start `(line-skip-horizontal (line-prefix-end ,marker))))
    `(if (line-starts-with-ascii-ci ,marker)
         (,close-paragraph
          (start-node ,(key-line-node rule))
          (token ,(key-line-key-token rule) start ,key-end)
          (token ,(key-line-trivia-token rule) ,key-end ,value-start)
          (token ,(key-line-value-token rule) ,value-start
                 (line-trim-end-from ,value-start))
          (token ,(key-line-trivia-token rule)
                 (line-trim-end-from ,value-start) end)
          (finish-node))
         (,otherwise))))

(def (declared-key-chain rule otherwise)
  (foldr (lambda (key next) (declared-key-form rule key next))
         otherwise (key-line-keys rule)))

(def (context-key-form)
  (declared-key-chain
   clock-rule
   `(if (state after-heading)
        (,(declared-key-chain planning-rule (keyword-form)))
        (,(keyword-form)))))

(def (headline-form)
  `(if (uint-positive? (line-marker-level ,heading-marker ,heading-separator))
       (,close-paragraph
        (close-through open-levels
                       (line-marker-level ,heading-marker ,heading-separator))
        (open-level open-levels
                    (line-marker-level ,heading-marker ,heading-separator)
                    OrgSection)
        (start-node OrgHeadline)
        (token HeadlineLine start
               (line-marker-end ,heading-marker ,heading-separator))
        (token HeadlineTrivia
               (line-marker-end ,heading-marker ,heading-separator)
               (line-skip-horizontal
                (line-marker-end ,heading-marker ,heading-separator)))
        (token HeadlineTitle
               (line-skip-horizontal
                (line-marker-end ,heading-marker ,heading-separator))
               (line-trim-end-from
                (line-skip-horizontal
                 (line-marker-end ,heading-marker ,heading-separator))))
        (token HeadlineTrivia
               (line-trim-end-from
                (line-skip-horizontal
                 (line-marker-end ,heading-marker ,heading-separator))) end)
        (finish-node)
        (set-bool after-heading (bool #t)))
       (,(context-key-form)
        (set-bool after-heading (bool #f)))))

(def (property-line-form)
  `(if (line-has-key-after-prefix? ,property-marker)
       ((start-node OrgNodeProperty)
        (token PropertyTrivia start (line-prefix-end ,property-marker))
        (token PropertyKey (line-prefix-end ,property-marker)
               (line-scan-key (line-prefix-end ,property-marker)))
        (token PropertyTrivia
               (line-scan-key (line-prefix-end ,property-marker))
               (line-skip-horizontal
                (line-step (line-scan-key (line-prefix-end ,property-marker)))))
        (token PropertyValue
               (line-skip-horizontal
                (line-step (line-scan-key (line-prefix-end ,property-marker))))
               (line-trim-end-from
                (line-skip-horizontal
                 (line-step (line-scan-key (line-prefix-end ,property-marker))))))
        (token PropertyTrivia
               (line-trim-end-from
                (line-skip-horizontal
                 (line-step (line-scan-key (line-prefix-end ,property-marker))))) end)
        (finish-node))
       ((token TextLine start end))))

(def (property-body-form)
  `(if (line-marker-ascii-ci ,property-close)
       ((token DrawerEndLine start end) (finish-node)
        (set-bool property-drawer-open (bool #f)))
       (,(property-line-form))))

(def (property-open-form otherwise)
  `(if (line-marker-ascii-ci ,property-open)
       (,close-paragraph (start-node OrgPropertyDrawer)
        (token DrawerBeginLine start end)
        (set-bool after-heading (bool #f))
        (set-bool property-drawer-open (bool #t)))
       (,otherwise)))

(def (block-header-forms rule)
  (let* ((opening (block-line-opening rule))
         (begin-token (block-line-begin-token rule))
         (header (block-line-header rule)))
    (if (not header)
      `((token ,begin-token start end))
      (let ((argument-token (block-header-argument-token header))
            (trivia-token (block-header-trivia-token header)))
        `((token ,begin-token start (line-prefix-end ,opening))
          (if (line-has-word-after-prefix? ,opening)
              ((token ,trivia-token
                      (line-prefix-end ,opening)
                      (line-skip-horizontal (line-prefix-end ,opening)))
               (token ,argument-token
                      (line-skip-horizontal (line-prefix-end ,opening))
                      (line-scan-word
                       (line-skip-horizontal (line-prefix-end ,opening))))
               (token ,trivia-token
                      (line-scan-word
                       (line-skip-horizontal (line-prefix-end ,opening))) end))
              ((token ,trivia-token (line-prefix-end ,opening) end))))))))

(def (opaque-open-form block-id otherwise)
  (let ((id (car block-id)) (rule (cdr block-id)))
    `(if (line-prefix-boundary-ascii-ci ,(block-line-opening rule))
         ,(append (list close-paragraph
                        (list 'start-node (block-line-block-node rule)))
                  (block-header-forms rule)
                  (list '(set-bool after-heading (bool #f))
                        `(set-uint active-opaque-block (uint ,id))))
         (,otherwise))))

(def (opaque-open-chain otherwise)
  (foldr opaque-open-form otherwise opaque-blocks))

(def (opaque-body-form block-id otherwise)
  (let ((id (car block-id)) (rule (cdr block-id)))
    `(if (uint-equal? (state active-opaque-block) (uint ,id))
         ((if (line-marker-ascii-ci ,(block-line-closing rule))
              ((token ,(block-line-end-token rule) start end)
               (finish-node) (set-uint active-opaque-block (uint 0)))
              ((token ,(block-line-body-token rule) start end))))
         (,otherwise))))

(def (opaque-body-chain)
  (foldr opaque-body-form '(token TextLine start end) opaque-blocks))

(def table-content-end '(line-content-end))
(def table-indent '(line-skip-horizontal start))
(def table-index '(line-index table-byte-index))
(def table-cell-start '(state-offset table-cell-start))
(def table-line-predicate
  `(line-byte-equal? ,table-indent ,table-byte))

(def (table-cell-forms until)
  `((start-node ,(table-line-cell-node table-rule))
    (token ,(table-line-cell-token table-rule) ,table-cell-start ,until)
    (finish-node)))

(def (table-row-form)
  `(if (and (line-bytes-all-in? ,table-indent ,table-content-end
                                 (,table-byte 43 45 58 9 32))
            (line-bytes-any-in? ,table-indent ,table-content-end (45)))
       ((start-node ,(table-line-rule-row-node table-rule))
        (token ,(table-line-rule-token table-rule) start end)
        (finish-node))
       ((start-node ,(table-line-row-node table-rule))
        (for-line-bytes table-byte-index ,table-indent ,table-content-end
          ((if (line-byte-equal? ,table-index 92)
               ((set-bool table-escaped (not (state table-escaped))))
               ((if (and (line-byte-equal? ,table-index ,table-byte)
                         (not (state table-escaped)))
                    ((if (state table-seen-separator)
                         ,(table-cell-forms table-index)
                         ((token ,(table-line-trivia-token table-rule)
                                 start ,table-index)))
                     (token ,(table-line-separator-token table-rule)
                            ,table-index (line-step ,table-index))
                     (set-uint table-cell-start
                               (offset (line-step ,table-index)))
                     (set-bool table-seen-separator (bool #t))) ())
                (set-bool table-escaped (bool #f))))))
        (if (state table-seen-separator)
            ((if (line-bytes-all-in? ,table-cell-start ,table-content-end
                                     (9 32))
                 ((token ,(table-line-trivia-token table-rule)
                         ,table-cell-start ,table-content-end))
                 ,(table-cell-forms table-content-end))) ())
        (token ,(table-line-trivia-token table-rule) ,table-content-end end)
        (finish-node)
        (set-bool table-seen-separator (bool #f))
        (set-bool table-escaped (bool #f)))))

(def (table-or-element-form)
  `(if ,table-line-predicate
       (,close-paragraph
        (if (not (state table-open))
            ((start-node ,(table-line-table-node table-rule))
             (set-bool table-open (bool #t))) ())
        ,(table-row-form)
        (set-bool after-heading (bool #f)))
       ((if (state table-open)
            ((finish-node) (set-bool table-open (bool #f))) ())
        ,(property-open-form
          (opaque-open-chain (headline-form))))))

(def org-event-initial
  '((open-levels (uint-stack)) (active-opaque-block 0)
    (property-drawer-open #f) (paragraph-open #f) (after-heading #f)
    (table-open #f) (table-seen-separator #f) (table-escaped #f)
    (table-cell-start 0)))

(def org-event-line-forms
  `((if (uint-positive? (state active-opaque-block))
        (,(opaque-body-chain))
        ((if (state property-drawer-open)
             (,(property-body-form))
             (,(table-or-element-form)))))))

(def org-event-finish-forms
  '((if (uint-positive? (state active-opaque-block)) ((finish-node)) ())
    (if (state property-drawer-open) ((finish-node)) ())
    (if (state table-open) ((finish-node)) ())
    (if (state paragraph-open) ((finish-node)) ())
    (close-all open-levels)))
