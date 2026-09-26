;;; -*- Gerbil -*-
;;; Org-owned, compositional event strategy. POO parser declarations own markers;
;;; the resulting forms execute in Scheme and AOT-lower to one Rust function.

(import (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-blocks
                 block-line-block-node block-line-opening block-line-closing
                 block-line-body-line block-line-begin-token
                 block-line-body-token block-line-end-token block-line-header
                 block-line-unclosed block-line-heading-bound block-line-indent
                 block-line-contents
                 key-value-line-marker
                 block-header-argument-token block-header-trivia-token)
        (only-in "../../parser.ss" org-v1-line-structure)
        (only-in "event-paragraph.ss"
                 paragraph-event-initial paragraph-close-form paragraph-finish-form
                 paragraph-line-form paragraph-event-helpers)
        (only-in "event-headline-tags.ss"
                 event-headline-tags-initial)
        (only-in "event-headline.ss"
                 headline-form heading-marker heading-separator
                 ascii-ci-pattern-at offset-after)
        (only-in "event-source-header.ss"
                 event-source-header-initial event-source-header-forms)
        (only-in "event-table.ss"
                 table-event-initial table-close-form table-or-element-form)
        (only-in "event-list.ss"
                 list-close-all list-close-paragraph-form list-or-element-form)
        (only-in "event-special-block.ss"
                 special-event-initial special-block-id special-future-scan
                 special-open-form special-close-form)
        (only-in "objects.ss"
                 make-org-event-block org-event-block-id org-event-block-rule))
(export org-event-initial org-event-line-forms org-event-finish-forms
        org-event-helpers)

(def (block-by-node node)
  (or (ormap (lambda (block)
               (and (eq? (block-line-block-node block) node) block))
             (line-structure-blocks org-v1-line-structure))
      (error "missing Org block declaration" node)))

(def property-rule (block-by-node 'OrgPropertyDrawer))
(def property-open (block-line-opening property-rule))
(def property-close (block-line-closing property-rule))

(def (numbered-blocks names)
  (let loop ((rest names) (id 1))
    (if (null? rest) '()
      (cons (make-org-event-block id (block-by-node (car rest)))
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
    ,heading-marker
    ,heading-separator
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
                            (uint ,(org-event-block-id parent)))
               ,(future-close-scan
                 rule (block-line-closing (org-event-block-rule parent))))
          ,otherwise))
   `(and (uint-equal? (stack-top container-frames) (uint 0))
         ,(future-close-scan rule ""))
   container-blocks))
(def (special-future-condition)
  (foldr
   (lambda (parent otherwise)
     `(or (and (uint-equal? (stack-top container-frames)
                            (uint ,(org-event-block-id parent)))
               ,(special-future-scan
                 (block-line-closing (org-event-block-rule parent))
                 heading-marker heading-separator))
          ,otherwise))
   `(and (or (uint-equal? (stack-top container-frames) (uint 0))
             (uint-equal? (stack-top container-frames)
                          (uint ,special-block-id)))
         ,(special-future-scan "" heading-marker heading-separator))
   container-blocks))
(def ascii-letter-bytes
  (map char->integer
       (string->list "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz")))
(def ascii-name-bytes
  (append ascii-letter-bytes
          (map char->integer (string->list "0123456789_-"))))

(def close-paragraph paragraph-close-form)

(def comment-marker '(line-skip-horizontal start))
(def comment-next `(line-step ,comment-marker))
(def comment-line?
  `(and (line-byte-equal? ,comment-marker 35)
        (or (line-byte-equal? ,comment-next 32)
            (line-bytes-all-in? ,comment-next (line-content-end) ()))))
(def close-comment
  '(if (state comment-open)
       ((finish-node) (set-bool comment-open (bool #f))) ()))

(def (comment-line-forms)
  '((if (not (state comment-open))
        ((start-node OrgComment) (set-bool comment-open (bool #t))) ())
    (token CommentLine start end)
    (set-bool after-heading (bool #f))))

(def paragraph-form paragraph-line-form)

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
            (trivia-token (block-header-trivia-token header))
            (argument-end
             `(line-scan-word
               (line-skip-horizontal (line-prefix-end ,opening)))))
        `((token ,begin-token start (line-prefix-end ,opening))
          (if (line-has-word-after-prefix? ,opening)
              ((token ,trivia-token
                      (line-prefix-end ,opening)
                      (line-skip-horizontal (line-prefix-end ,opening)))
               (token ,argument-token
                      (line-skip-horizontal (line-prefix-end ,opening))
                      ,argument-end)
               ,@(if (eq? (block-line-block-node rule) 'OrgSourceBlock)
                   (event-source-header-forms argument-end)
                   `((token ,trivia-token ,argument-end end))))
              ((token ,trivia-token (line-prefix-end ,opening) end))))))))

(def (opaque-open-form block-id otherwise)
  (let ((id (org-event-block-id block-id))
        (rule (org-event-block-rule block-id)))
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
  (let ((id (org-event-block-id block-id))
        (rule (org-event-block-rule block-id)))
    `(if (uint-equal? (state active-opaque-block) (uint ,id))
         ((if (line-marker-ascii-ci ,(block-line-closing rule))
              ((token ,(block-line-end-token rule) start end)
               (finish-node) (set-uint active-opaque-block (uint 0)))
              ((token ,(block-line-body-token rule) start end))))
         (,otherwise))))

(def (opaque-body-chain)
  (foldr opaque-body-form '(token TextLine start end) opaque-blocks))

(def container-indent '(line-skip-horizontal start))

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
  (let ((id (org-event-block-id block-id))
        (rule (org-event-block-rule block-id)))
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
  (let ((id (org-event-block-id block-id))
        (rule (org-event-block-rule block-id)))
    `(if (and (stack-nonempty? container-frames)
              (uint-equal? (stack-top container-frames) (uint ,id))
              ,(container-close-condition rule))
         (,close-paragraph
          ,@list-close-all
          ,fixed-width-close
          ,table-close-form
          (token ,(block-line-end-token rule) start end)
          (close-frame container-frames 1)
          (set-bool container-closed (bool #t)))
         ,otherwise)))

(def (container-close-chain)
  (let (chain (foldr (lambda (block rest)
                       (list (container-close-form block rest)))
                     '() container-blocks))
    `((if (stack-nonempty? container-frames)
          ,chain ()))))

(def horizontal-rule-start '(line-skip-horizontal start))
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

(def (diary-sexp-form otherwise)
  `(if (line-starts-with "%%(")
       (,close-paragraph
        ,fixed-width-close
        (start-node OrgDiarySexp)
        (token DiarySexpValue start (line-content-end))
        (token DiarySexpTrivia (line-content-end) end)
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

(def (container-or-opaque-form)
  `(,@(container-open-chain)
    (if (state container-opened)
        ((set-bool container-opened (bool #f)))
        (,@(opaque-open-chain)
         (if (uint-positive? (state active-opaque-block))
             () (,(special-open-form close-paragraph
                                     (special-future-condition))
                 (if (state container-opened)
                     ((set-bool container-opened (bool #f)))
                     (,(headline-form)))))))))

(def (non-table-element-form)
  `(if ,comment-line?
       (,fixed-width-close
        ,close-paragraph
        ,@(comment-line-forms))
       (,close-comment
        ,(diary-sexp-form
          (list (fixed-width-form
                 (list (horizontal-rule-form
                        (list (property-open-form
                               (container-or-opaque-form)))))))))))

(def (org-table-or-element-form)
  (table-or-element-form close-paragraph fixed-width-close
                         (non-table-element-form)))


(def footnote-label-start '(line-prefix-end "[fn:"))
(def footnote-label-end `(line-scan-key ,footnote-label-start))
(def footnote-content-start `(line-step ,footnote-label-end))
(def footnote-definition?
  `(and (uint-equal? (stack-top container-frames) (uint 0))
        (line-starts-with "[fn:")
        (offset-less? ,footnote-label-start ,footnote-label-end)
        (line-byte-equal? ,footnote-label-end 93)))

(def (footnote-pending-trivia (reset? #t))
  (append
   '((start-node OrgTextLine)
     (token TextLine (state-offset footnote-blank-start)
            (state-offset footnote-blank-end))
     (finish-node))
   (if reset? '((set-bool footnote-blank-pending (bool #f))) '())))

(def (footnote-close-forms (reset-open? #t) (pending-inside? #t))
  `(,close-paragraph
    ,@list-close-all
    ,table-close-form
    ,fixed-width-close
    ,close-comment
    (close-all-frames container-frames 1)
    ,@(if pending-inside?
        `((if (state footnote-blank-pending)
              ,(footnote-pending-trivia) ())) '())
    (finish-node)
    ,@(if reset-open? '((set-bool footnote-open (bool #f))) '())
    ,@(if pending-inside? '()
        `((if (state footnote-blank-pending)
              ,(footnote-pending-trivia) ())))))

(def (footnote-prepare-open-forms)
  `(,close-paragraph
    ,@list-close-all
    ,table-close-form
    ,fixed-width-close
    ,close-comment))

(def (footnote-open-forms)
  `((start-node OrgFootnoteDefinition)
    (token FootnoteDefinitionDelimiter start ,footnote-label-start)
    (token FootnoteDefinitionLabel ,footnote-label-start ,footnote-label-end)
    (token FootnoteDefinitionDelimiter ,footnote-label-end
           ,footnote-content-start)
    (if (line-bytes-all-in? ,footnote-content-start
                            (line-content-end) (9 32))
        ((token FootnoteDefinitionDelimiter ,footnote-content-start end))
        ((start-node OrgParagraph)
         (set-bool paragraph-open (bool #t))
         (set-uint paragraph-start (offset ,footnote-content-start))
         (set-uint paragraph-end (offset end))))
    (set-bool footnote-open (bool #t))
    (set-bool after-heading (bool #f))))

(def (footnote-or-element-forms)
  `((if ,footnote-definition?
        ((if (state footnote-open)
             ,(footnote-close-forms #f)
             ,(footnote-prepare-open-forms))
         ,@(footnote-open-forms)
         (set-bool footnote-line-handled (bool #t)))
        ((if (state footnote-open)
             ((if (line-blank?)
                  ((if (state footnote-blank-pending)
                       ,(footnote-close-forms #t #f)
                       ((set-uint footnote-blank-start (offset start))
                        (set-uint footnote-blank-end (offset end))
                        (set-bool footnote-blank-pending (bool #t))
                        (set-bool footnote-line-handled (bool #t)))))
                  ((if (state footnote-blank-pending)
                       ((if (state paragraph-open)
                            ((set-bool paragraph-post-blank (bool #t))
                             (set-uint paragraph-blank-end
                                       (offset (state-offset footnote-blank-end)))
                             (set-bool footnote-blank-pending (bool #f)))
                            ,(footnote-pending-trivia))) ()))))
             ())))
    (if (not (state footnote-line-handled))
        ,(list-or-element-form table-close-form fixed-width-close close-paragraph
                               comment-line? comment-line-forms
                               org-table-or-element-form) ())
    (set-bool footnote-line-handled (bool #f))))

(def (headline-or-element-form)
  `(if (and (uint-equal? (stack-top container-frames) (uint 0))
            (uint-positive? (line-marker-level ,heading-marker
                                               ,heading-separator)))
       ((if (state footnote-open)
            ,(footnote-close-forms)
            (,@list-close-all
             ,table-close-form
             ,fixed-width-close
             ,close-comment))
        ,(headline-form))
       ,(footnote-or-element-forms)))

(def org-event-initial
  (append
   '((open-levels (uint-stack)) (active-opaque-block 0)
    (property-drawer-open #f) (after-heading #f)
    (comment-open #f)
    (fixed-width-open #f))
   table-event-initial
   '((planning-value-start 0) (planning-value-end 0)
    (container-frames (uint-stack)) (container-closed #f)
    (container-opened #f)
    (list-frames (uint-stack)) (list-present #f) (list-ordered #f)
    (list-column 0) (list-bullet-start 0) (list-bullet-end 0)
    (list-content-start 0) (list-paragraph-open #f) (list-blank-count 0)
    (list-paragraph-start 0) (list-paragraph-end 0))
   '((footnote-open #f) (footnote-line-handled #f)
     (footnote-blank-pending #f)
     (footnote-blank-start 0) (footnote-blank-end 0))
   special-event-initial
   paragraph-event-initial
   event-headline-tags-initial
   event-source-header-initial))

(def org-event-helpers paragraph-event-helpers)

(def org-event-line-forms
  `((if (uint-positive? (state active-opaque-block))
        (,(opaque-body-chain))
        ((if (state property-drawer-open)
             (,(property-body-form))
             ((if (state comment-open)
                  ((if ,comment-line? () (,close-comment))) ())
              ,(special-close-form close-paragraph list-close-all
                                   fixed-width-close table-close-form)
              ,@(container-close-chain)
              (if (state container-closed)
                  ((set-bool container-closed (bool #f)))
                  (,(headline-or-element-form)))))))))

(def org-event-finish-forms
  `((if (uint-positive? (state active-opaque-block)) ((finish-node)) ())
    (if (state property-drawer-open) ((finish-node)) ())
    (if (state table-open) ((finish-node)) ())
    ,(list-close-paragraph-form #f)
    (close-all-frames list-frames 2)
    (if (state fixed-width-open) ((finish-node)) ())
    ,paragraph-finish-form
    (if (state comment-open) ((finish-node)) ())
    (close-all-frames container-frames 1)
    (if (state footnote-open)
        ((if (state footnote-blank-pending)
             ,(footnote-pending-trivia #f) ())
         (finish-node)) ())
    (close-all open-levels)))
