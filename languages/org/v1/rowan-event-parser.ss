;;; -*- Gerbil -*-
;;; Org-owned contextual event algorithm compiled by the generic Rowan AOT path.
;;; This replaces the flat line-event fixture; full Element cutover is pending.

(import (only-in :gerbil-parser/rust-rowan-event-support
                 run-event-fold event-fold-ir-json)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-heading line-structure-blocks
                 line-structure-key-lines key-line-node key-line-prefix
                 key-line-keys key-line-separator
                 heading-line-marker heading-line-separator
                 block-line-block-node block-line-opening block-line-closing
                 block-line-body-line key-value-line-marker)
        (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "parser.ss" org-v1-line-structure))
(export parse-org-rowan-events parse_org_rowan_events)

(def (org-block-by-node node)
  (or (ormap (lambda (block)
               (and (eq? (block-line-block-node block) node) block))
             (line-structure-blocks org-v1-line-structure))
      (error "missing Org block declaration" node)))

(def (org-key-line-by-node node)
  (or (ormap (lambda (rule) (and (eq? (key-line-node rule) node) rule))
             (line-structure-key-lines org-v1-line-structure))
      (error "missing Org key-line declaration" node)))

(def org-source-block-rule (org-block-by-node 'OrgSourceBlock))
(def org-property-drawer-rule (org-block-by-node 'OrgPropertyDrawer))
(def org-heading-rule (line-structure-heading org-v1-line-structure))
(def org-keyword-rule (org-key-line-by-node 'OrgKeyword))
(def org-babel-call-rule (org-key-line-by-node 'OrgBabelCall))
(def org-source-open (block-line-opening org-source-block-rule))
(def org-source-close (block-line-closing org-source-block-rule))
(def org-property-open (block-line-opening org-property-drawer-rule))
(def org-property-close (block-line-closing org-property-drawer-rule))
(def org-heading-marker (heading-line-marker org-heading-rule))
(def org-heading-separator (heading-line-separator org-heading-rule))
(def org-keyword-prefix (key-line-prefix org-keyword-rule))
(def org-babel-call-marker
  (let (keys (key-line-keys org-babel-call-rule))
    (unless (and (pair? keys) (null? (cdr keys)))
      (error "Org Babel CALL requires one declared key" keys))
    (string-append (key-line-prefix org-babel-call-rule)
                   (car keys) (key-line-separator org-babel-call-rule))))
(def org-property-marker
  (key-value-line-marker (block-line-body-line org-property-drawer-rule)))

(def org-event-initial
  '((open-levels (uint-stack)) (source-block-open #f)
    (property-drawer-open #f) (paragraph-open #f)))

(def org-event-line-forms
  `((if (state source-block-open)
       ((if (line-marker-ascii-ci ,org-source-close)
            ((token BlockEndLine start end) (finish-node)
             (set-bool source-block-open (bool #f)))
            ((token TextLine start end))))
       ((if (state property-drawer-open)
            ((if (line-marker-ascii-ci ,org-property-close)
                 ((token DrawerEndLine start end) (finish-node)
                  (set-bool property-drawer-open (bool #f)))
                 ((if (line-has-key-after-prefix? ,org-property-marker)
                      ((start-node OrgNodeProperty)
                       (token PropertyTrivia start
                              (line-prefix-end ,org-property-marker))
                       (token PropertyKey (line-prefix-end ,org-property-marker)
                              (line-scan-key
                               (line-prefix-end ,org-property-marker)))
                       (token PropertyTrivia
                              (line-scan-key
                               (line-prefix-end ,org-property-marker))
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key
                                 (line-prefix-end ,org-property-marker)))))
                       (token PropertyValue
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key
                                 (line-prefix-end ,org-property-marker))))
                              (line-trim-end))
                       (token PropertyTrivia (line-trim-end) end)
                       (finish-node))
                      ((token TextLine start end))))))
            ((if (line-marker-ascii-ci ,org-property-open)
                 ((if (state paragraph-open)
                      ((finish-node)
                       (set-bool paragraph-open (bool #f))) ())
                  (start-node OrgPropertyDrawer)
                  (token DrawerBeginLine start end)
                  (set-bool property-drawer-open (bool #t)))
                 ((if (line-prefix-boundary-ascii-ci ,org-source-open)
            ((if (state paragraph-open)
                 ((finish-node) (set-bool paragraph-open (bool #f))) ())
             (start-node OrgSourceBlock)
             (token BlockBeginLine start (line-prefix-end ,org-source-open))
             (if (line-has-word-after-prefix? ,org-source-open)
                 ((token BlockHeaderTrivia
                         (line-prefix-end ,org-source-open)
                         (line-skip-horizontal
                          (line-prefix-end ,org-source-open)))
                  (token SourceLanguage
                         (line-skip-horizontal
                          (line-prefix-end ,org-source-open))
                         (line-scan-word
                          (line-skip-horizontal
                           (line-prefix-end ,org-source-open))))
                  (token BlockHeaderTrivia
                         (line-scan-word
                          (line-skip-horizontal
                           (line-prefix-end ,org-source-open))) end))
                 ((token BlockHeaderTrivia
                         (line-prefix-end ,org-source-open) end)))
             (set-bool source-block-open (bool #t)))
            ((if (uint-positive? (line-marker-level ,org-heading-marker
                                                    ,org-heading-separator))
                 ((if (state paragraph-open)
                      ((finish-node) (set-bool paragraph-open (bool #f))) ())
                  (close-through open-levels
                                 (line-marker-level ,org-heading-marker
                                                    ,org-heading-separator))
                  (open-level open-levels
                              (line-marker-level ,org-heading-marker
                                                 ,org-heading-separator)
                              OrgSection)
                  (start-node OrgHeadline)
                  (token HeadlineLine start
                         (line-marker-end ,org-heading-marker
                                          ,org-heading-separator))
                  (token HeadlineTrivia
                         (line-marker-end ,org-heading-marker
                                          ,org-heading-separator)
                         (line-skip-horizontal
                          (line-marker-end ,org-heading-marker
                                           ,org-heading-separator)))
                  (token HeadlineTitle
                         (line-skip-horizontal
                          (line-marker-end ,org-heading-marker
                                           ,org-heading-separator))
                         (line-trim-end))
                  (token HeadlineTrivia (line-trim-end) end)
                  (finish-node))
                 ((if (line-has-key-after-prefix? ,org-keyword-prefix)
                      ((if (state paragraph-open)
                           ((finish-node)
                            (set-bool paragraph-open (bool #f))) ())
                       (if (line-starts-with-ascii-ci ,org-babel-call-marker)
                           ((start-node OrgBabelCall))
                           ((start-node OrgKeyword)))
                       (token KeywordTrivia start
                              (line-prefix-end ,org-keyword-prefix))
                       (token KeywordKey (line-prefix-end ,org-keyword-prefix)
                              (line-scan-key
                               (line-prefix-end ,org-keyword-prefix)))
                       (token KeywordTrivia
                              (line-scan-key
                               (line-prefix-end ,org-keyword-prefix))
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key
                                 (line-prefix-end ,org-keyword-prefix)))))
                       (token KeywordValue
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key
                                 (line-prefix-end ,org-keyword-prefix))))
                              (line-trim-end))
                       (token KeywordTrivia (line-trim-end) end)
                       (finish-node))
                      ((if (line-blank?)
                           ((if (state paragraph-open)
                                ((finish-node)
                                 (set-bool paragraph-open (bool #f))) ())
                            (start-node OrgTextLine)
                            (token TextLine start end) (finish-node))
                           ((if (not (state paragraph-open))
                                ((start-node OrgParagraph)
                                 (set-bool paragraph-open (bool #t))) ())
                            (start-node OrgTextLine)
                            (token TextLine start end) (finish-node)))))))))))))))))

(def org-event-finish-forms
  '((if (state source-block-open) ((finish-node)) ())
   (if (state property-drawer-open) ((finish-node)) ())
   (if (state paragraph-open) ((finish-node)) ())
   (close-all open-levels)))

(def (parse-org-rowan-events source)
  (run-event-fold source 'OrgFile org-event-initial
                  org-event-line-forms org-event-finish-forms))

(def parse_org_rowan_events
  (event-fold-ir-json 'parse_org_rowan_events org-v1-language-grammar
                      'OrgFile org-event-initial
                      org-event-line-forms org-event-finish-forms))
