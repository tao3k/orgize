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
                 block-line-unclosed block-line-heading-bound block-line-indent
                 block-line-contents
                 key-value-line-marker
                 block-header-argument-token block-header-trivia-token
                 line-structure-table
                 table-line-delimiter table-line-table-node
                 table-line-row-node table-line-rule-row-node
                 table-line-cell-node table-line-separator-token
                 table-line-cell-token table-line-trivia-token
                 table-line-rule-token line-structure-list
                 list-line-unordered-markers list-line-ordered
                 list-line-tab-width list-line-list-node
                 list-line-item-node list-line-bullet-token
                 list-line-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "event-inline.ss" event-inline-initial event-text-line-forms))
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
(def list-rule (line-structure-list org-v1-line-structure))
(def table-byte
  (char->integer (string-ref (table-line-delimiter table-rule) 0)))
(def property-open (block-line-opening property-rule))
(def property-close (block-line-closing property-rule))
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

(def container-blocks
  (numbered-blocks
   '(OrgDynamicBlock OrgDrawer OrgQuoteBlock OrgVerseBlock OrgCenterBlock)))

(def (future-close-scan rule stop)
  `(future-line-marker-before-boundary?
    ,(block-line-closing rule) ,stop
    ,(heading-line-marker heading-rule)
    ,(heading-line-separator heading-rule)
    ,(block-line-indent rule)
    ,(eq? (block-line-contents rule) 'elements)
    ,(let (body (block-line-body-line rule))
       (if body (key-value-line-marker body) ""))))

(def (future-close-condition rule)
  (unless (and (eq? (block-line-unclosed rule) 'recover-as-text)
               (block-line-heading-bound rule))
    (error "Org event block requires declared text recovery and heading boundary"
           (block-line-block-node rule)))
  (foldr
   (lambda (parent otherwise)
     `(or (and (uint-equal? (stack-top container-frames)
                            (uint ,(car parent)))
               ,(future-close-scan rule (block-line-closing (cdr parent))))
          ,otherwise))
   `(and (uint-equal? (stack-top container-frames) (uint 0))
         ,(future-close-scan rule ""))
   container-blocks))
(def ascii-letter-bytes
  (map char->integer
       (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")))
(def ascii-name-bytes
  (append ascii-letter-bytes
          (map char->integer (string->list "0123456789_-"))))

(def close-paragraph
  '(if (state paragraph-open)
       ((finish-node) (set-bool paragraph-open (bool #f))) ()))

(def (paragraph-form)
  `(if (line-blank?)
       (,close-paragraph
        (start-node OrgTextLine) (token TextLine start end) (finish-node))
       ((if (not (state paragraph-open))
            ((start-node OrgParagraph) (set-bool paragraph-open (bool #t))) ())
        ,@(event-text-line-forms 'start))))

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
  `(if (and (uint-equal? (stack-top container-frames) (uint 0))
            (uint-positive? (line-marker-level ,heading-marker ,heading-separator)))
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

(def property-indent '(line-skip-horizontal start))
(def property-key-start `(line-step ,property-indent))
(def property-key-end
  `(line-scan-nonspace-until ,property-key-start ":"))
(def property-value-start
  `(line-skip-horizontal (line-step ,property-key-end)))
(def property-value-end `(line-trim-end-from ,property-value-start))

(def (property-line-form)
  `(if (and (line-byte-equal? ,property-indent 58)
            (offset-less? ,property-key-start ,property-key-end)
            (line-byte-equal? ,property-key-end 58))
       ((start-node OrgNodeProperty)
        (token PropertyTrivia start ,property-key-start)
        (token PropertyKey ,property-key-start ,property-key-end)
        (token PropertyTrivia ,property-key-end ,property-value-start)
        (token PropertyValue ,property-value-start ,property-value-end)
        (token PropertyTrivia ,property-value-end end)
        (finish-node))
       ((token TextLine start end))))

(def (property-body-form)
  `(if ,(container-close-condition property-rule)
       ((token DrawerEndLine start end) (finish-node)
        (set-bool property-drawer-open (bool #f)))
       (,(property-line-form))))

(def (property-open-form otherwise)
  `(if (and ,(ascii-ci-pattern-at property-indent property-open)
            (line-bytes-all-in?
             ,(offset-after property-indent (string-length property-open))
             end (9 10 13 32))
            ,(future-close-condition property-rule))
       (,close-paragraph (start-node OrgPropertyDrawer)
        (token DrawerBeginLine start end)
        (set-bool after-heading (bool #f))
        (set-bool property-drawer-open (bool #t)))
       ,otherwise))

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
    `(if (and (line-prefix-boundary-ascii-ci ,(block-line-opening rule))
              ,(future-close-condition rule))
         ,(append (list close-paragraph
                        (list 'start-node (block-line-block-node rule)))
                  (block-header-forms rule)
                  (list '(set-bool after-heading (bool #f))
                        `(set-uint active-opaque-block (uint ,id))))
         ,otherwise)))

(def (opaque-open-chain)
  (let (chain (foldr (lambda (block rest)
                       (list (opaque-open-form block rest)))
                     '() opaque-blocks))
    `((if (line-byte-equal? (line-skip-horizontal start) 35)
          ,chain ()))))

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

(def container-indent '(line-skip-horizontal start))

(def (offset-after from count)
  (let loop ((offset from) (remaining count))
    (if (= remaining 0) offset
      (loop `(line-step ,offset) (- remaining 1)))))

(def (ascii-ci-pattern-at from pattern)
  (let loop ((rest (string->list pattern)) (offset from) (predicates []))
    (if (null? rest)
      (cons 'and (reverse predicates))
      (let* ((upper (char->integer (char-upcase (car rest))))
             (lower (char->integer (char-downcase (car rest))))
             (check (if (= upper lower)
                      `(line-byte-equal? ,offset ,upper)
                      `(line-bytes-any-in? ,offset (line-step ,offset)
                                           (,upper ,lower)))))
        (loop (cdr rest) `(line-step ,offset) (cons check predicates))))))

(def (container-marker-end rule)
  (offset-after container-indent (string-length (block-line-opening rule))))

(def (container-close-condition rule)
  (let ((closing (block-line-closing rule))
        (indent container-indent))
    `(and ,(ascii-ci-pattern-at indent closing)
          (line-bytes-all-in? ,(offset-after indent (string-length closing))
                              end (9 10 13 32)))))

(def (named-container-start rule)
  `(line-skip-horizontal ,(container-marker-end rule)))

(def (container-open-condition rule)
  (let ((opening (block-line-opening rule)))
    (cond
     ((eq? (block-line-block-node rule) 'OrgDrawer)
      (let* ((name-start '(line-step (line-skip-horizontal start)))
             (name-end `(line-scan-key ,name-start))
             (after-colon `(line-step ,name-end)))
        `(and (line-byte-equal? ,container-indent 58)
              (not (line-marker-ascii-ci ,property-open))
              (line-bytes-any-in? ,name-start (line-step ,name-start)
                                  ,ascii-letter-bytes)
              (line-byte-equal? ,name-end 58)
              (line-bytes-all-in? ,after-colon end (9 10 13 32)))))
     ((eq? (block-line-block-node rule) 'OrgDynamicBlock)
      (let ((name-start (named-container-start rule))
            (prefix-end (container-marker-end rule)))
        `(and ,(ascii-ci-pattern-at container-indent opening)
              (line-bytes-any-in? ,prefix-end (line-step ,prefix-end) (9 32))
              (line-bytes-any-in? ,name-start (line-step ,name-start)
                                  ,ascii-letter-bytes)
              (line-bytes-all-in? ,name-start (line-scan-word ,name-start)
                                  ,ascii-name-bytes))))
     (else
      (let (prefix-end (container-marker-end rule))
        `(and ,(ascii-ci-pattern-at container-indent opening)
              (or (line-bytes-all-in? ,prefix-end end (9 10 13 32))
                  (line-bytes-any-in? ,prefix-end (line-step ,prefix-end)
                                      (9 32)))))))))

(def (container-header-forms rule)
  (if (eq? (block-line-block-node rule) 'OrgDrawer)
    (let* ((name-start '(line-step (line-skip-horizontal start)))
           (name-end `(line-scan-key ,name-start)))
      `((token ,(block-line-begin-token rule) start ,name-start)
        (token ,(block-header-argument-token (block-line-header rule))
               ,name-start ,name-end)
        (token ,(block-header-trivia-token (block-line-header rule))
               ,name-end end)))
    (if (eq? (block-line-block-node rule) 'OrgDynamicBlock)
      (let* ((prefix-end (container-marker-end rule))
             (name-start (named-container-start rule))
             (name-end `(line-scan-word ,name-start))
             (header (block-line-header rule)))
        `((token ,(block-line-begin-token rule) start ,prefix-end)
          (token ,(block-header-trivia-token header)
                 ,prefix-end ,name-start)
          (token ,(block-header-argument-token header)
                 ,name-start ,name-end)
          (token ,(block-header-trivia-token header) ,name-end end)))
      (block-header-forms rule))))

(def (container-open-form block-id otherwise)
  (let ((id (car block-id)) (rule (cdr block-id)))
    `(if (and ,(container-open-condition rule)
              ,(future-close-condition rule))
         ,(append (list close-paragraph
                        (list 'start-node (block-line-block-node rule)))
                  (container-header-forms rule)
                  (list `(push-frame container-frames (uint ,id))
                        '(set-bool after-heading (bool #f))
                        '(set-bool container-opened (bool #t))))
         ,otherwise)))

(def (container-open-chain)
  (let* ((chain (foldr (lambda (block rest)
                         (list (container-open-form block rest)))
                       '() container-blocks))
         (drawer (block-by-node 'OrgDrawer)))
    `((if (or (line-byte-equal? (line-skip-horizontal start) 35)
              (line-byte-equal? (line-skip-horizontal start) 58))
          ((if ,(container-close-condition drawer) () ,chain)) ()))))

(def (container-close-form block-id otherwise)
  (let ((id (car block-id)) (rule (cdr block-id)))
    `(if (and (stack-nonempty? container-frames)
              (uint-equal? (stack-top container-frames) (uint ,id))
              ,(container-close-condition rule))
         (,close-paragraph
          ,@list-close-all
          ,fixed-width-close
          (if (state table-open)
              ((finish-node) (set-bool table-open (bool #f))) ())
          (token ,(block-line-end-token rule) start end)
          (close-frames-while container-frames
                              (uint-equal? (stack-top container-frames)
                                           (uint ,id)) 1)
          (set-bool container-closed (bool #t)))
         ,otherwise)))

(def (container-close-chain)
  (let (chain (foldr (lambda (block rest)
                       (list (container-close-form block rest)))
                     '() container-blocks))
    `((if (stack-nonempty? container-frames)
          ,chain ()))))

(def table-content-end '(line-content-end))
(def table-indent '(line-skip-horizontal start))
(def table-index '(line-index table-byte-index))
(def table-cell-start '(state-offset table-cell-start))
(def table-line-predicate
  `(line-byte-equal? ,table-indent ,table-byte))

(def horizontal-rule-start table-indent)
(def horizontal-rule-condition
  `(and ,@(let loop ((index 0) (offset horizontal-rule-start))
            (if (= index 5) '()
              (cons `(line-byte-equal? ,offset 45)
                    (loop (+ index 1) `(line-step ,offset)))))
        (line-bytes-all-in? ,(offset-after horizontal-rule-start 5)
                            (line-content-end) (9 32 45))))

(def (horizontal-rule-form otherwise)
  `(if ,horizontal-rule-condition
       (,close-paragraph
        (start-node OrgHorizontalRule)
        (token HorizontalRuleLine start end)
        (finish-node)
        (set-bool after-heading (bool #f)))
       ,otherwise))

(def fixed-width-marker '(line-skip-horizontal start))
(def fixed-width-content-start `(line-step ,fixed-width-marker))
(def fixed-width-line?
  `(and (line-byte-equal? ,fixed-width-marker 58)
        (or (line-byte-equal? ,fixed-width-content-start 32)
            (line-bytes-all-in? ,fixed-width-content-start
                                (line-content-end) (9 32)))))
(def fixed-width-close
  '(if (state fixed-width-open)
       ((finish-node) (set-bool fixed-width-open (bool #f))) ()))

(def (fixed-width-form otherwise)
  `(if ,fixed-width-line?
       (,close-paragraph
        (if (not (state fixed-width-open))
            ((start-node OrgFixedWidth)
             (set-bool fixed-width-open (bool #t))) ())
        (token FixedWidthLine start end)
        (set-bool after-heading (bool #f)))
       (,fixed-width-close ,@otherwise)))

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

(def (container-or-opaque-form)
  `(,@(container-open-chain)
    (if (state container-opened)
        ((set-bool container-opened (bool #f)))
        (,@(opaque-open-chain)
         (if (uint-positive? (state active-opaque-block))
             () (,(headline-form)))))))

(def (non-table-element-form)
  (fixed-width-form
   (list (horizontal-rule-form
          (list (property-open-form (container-or-opaque-form)))))))

(def (table-or-element-form)
  `(if ,table-line-predicate
       (,fixed-width-close ,close-paragraph
        (if (not (state table-open))
            ((start-node ,(table-line-table-node table-rule))
             (set-bool table-open (bool #t))) ())
        ,(table-row-form)
        (set-bool after-heading (bool #f)))
       ((if (state table-open)
            ((finish-node) (set-bool table-open (bool #f))) ())
        ,(non-table-element-form))))

(def list-column '(state list-column))
(def list-top '(uint-divide (stack-top list-frames) (uint 2)))
(def list-frame-base `(uint-multiply ,list-column (uint 2)))
(def list-marker-form
  `(scan-list-marker ,(list-line-unordered-markers list-rule)
                     ,(list-line-ordered list-rule)
                     ,(list-line-tab-width list-rule)
                     list-present list-column list-ordered
                     list-bullet-start list-bullet-end list-content-start))

(def (list-frame-value ordered?)
  (if ordered? `(uint-add ,list-frame-base (uint 1)) list-frame-base))

(def list-close-paragraph
  '(if (state list-paragraph-open)
       ((finish-node) (set-bool list-paragraph-open (bool #f))) ()))

(def list-close-all
  `(,list-close-paragraph
    (close-all-frames list-frames 2)
    (set-uint list-blank-count (uint 0))))

(def (list-text-line from)
  `((if (not (state list-paragraph-open))
        ((start-node OrgParagraph)
         (set-bool list-paragraph-open (bool #t))) ())
    ,@(event-text-line-forms from)))

(def (list-marker-body ordered?)
  (let ((frame (list-frame-value ordered?))
        (list-node (list-line-list-node list-rule))
        (item-node (list-line-item-node list-rule))
        (trivia (list-line-trivia-token list-rule))
        (bullet (list-line-bullet-token list-rule)))
    `((close-frames-while list-frames
        (or (uint-greater? ,list-top ,list-column)
            (and (uint-equal? ,list-top ,list-column)
                 (uint-not-equal? (stack-top list-frames) ,frame))) 2)
      (if (and (stack-nonempty? list-frames)
               (uint-equal? ,list-top ,list-column))
          ((finish-node))
          ((start-node ,list-node)
           (push-frame list-frames ,frame)))
      (start-node ,item-node)
      (token ,trivia start (state-offset list-bullet-start))
      (token ,bullet (state-offset list-bullet-start)
             (state-offset list-bullet-end))
      (token ,trivia (state-offset list-bullet-end)
             (state-offset list-content-start))
      (if (offset-less? (state-offset list-content-start)
                        (line-content-end))
          ,(list-text-line '(state-offset list-content-start))
          ((token ,trivia (state-offset list-content-start) end)))
      (set-uint list-blank-count (uint 0)))))

(def (list-marker-forms)
  `((if (state table-open)
        ((finish-node) (set-bool table-open (bool #f))) ())
    ,fixed-width-close
    ,close-paragraph
    ,list-close-paragraph
    (if (state list-ordered)
        ,(list-marker-body #t)
        ,(list-marker-body #f))
    (set-bool after-heading (bool #f))))

(def (list-or-element-form)
  `(,list-marker-form
    (if (and (state list-present)
             (uint-equal?
              (line-marker-level ,heading-marker ,heading-separator) (uint 0)))
        ,(list-marker-forms)
        ((if (stack-nonempty? list-frames)
             ((if (line-blank?)
                  ((if (uint-equal? (state list-blank-count) (uint 0))
                       (,list-close-paragraph
                        (token ,(list-line-trivia-token list-rule) start end)
                        (set-uint list-blank-count (uint 1)))
                       (,@list-close-all ,(table-or-element-form))))
                  ((if (uint-greater?
                        (line-indent-column ,(list-line-tab-width list-rule))
                        ,list-top)
                       (,@(list-text-line 'start)
                        (set-uint list-blank-count (uint 0)))
                       (,@list-close-all ,(table-or-element-form))))))
             (,(table-or-element-form)))))))

(def org-event-initial
  (append
   '((open-levels (uint-stack)) (active-opaque-block 0)
    (property-drawer-open #f) (paragraph-open #f) (after-heading #f)
    (fixed-width-open #f)
    (table-open #f) (table-seen-separator #f) (table-escaped #f)
    (table-cell-start 0)
    (container-frames (uint-stack)) (container-closed #f)
    (container-opened #f)
    (list-frames (uint-stack)) (list-present #f) (list-ordered #f)
    (list-column 0) (list-bullet-start 0) (list-bullet-end 0)
    (list-content-start 0) (list-paragraph-open #f) (list-blank-count 0))
   event-inline-initial))

(def org-event-line-forms
  `((if (uint-positive? (state active-opaque-block))
        (,(opaque-body-chain))
        ((if (state property-drawer-open)
             (,(property-body-form))
             (,@(container-close-chain)
              (if (state container-closed)
                  ((set-bool container-closed (bool #f)))
                  (,@(list-or-element-form)))))))))

(def org-event-finish-forms
  '((if (uint-positive? (state active-opaque-block)) ((finish-node)) ())
    (if (state property-drawer-open) ((finish-node)) ())
    (if (state table-open) ((finish-node)) ())
    (if (state list-paragraph-open) ((finish-node)) ())
    (close-all-frames list-frames 2)
    (if (state fixed-width-open) ((finish-node)) ())
    (if (state paragraph-open) ((finish-node)) ())
    (close-all-frames container-frames 1)
    (close-all open-levels)))
