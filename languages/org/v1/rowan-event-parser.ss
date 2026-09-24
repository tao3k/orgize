;;; -*- Gerbil -*-
;;; Org-owned contextual event algorithm compiled by the generic Rowan AOT path.
;;; This replaces the flat line-event fixture; full Element cutover is pending.

(import (only-in :gerbil-parser/rust-rowan-event-support
                 define-event-fold-parser)
        (only-in "grammar.ss" org-v1-language-grammar))
(export parse-org-rowan-events parse_org_rowan_events)

(define-event-fold-parser
  parse-org-rowan-events parse_org_rowan_events org-v1-language-grammar OrgFile
  source ((open-levels (uint-stack)) (source-block-open #f)
          (property-drawer-open #f)
          (paragraph-open #f))
  ((if (state source-block-open)
       ((if (line-starts-with-ascii-ci "#+end_src")
            ((token BlockEndLine start end) (finish-node)
             (set-bool source-block-open (bool #f)))
            ((token TextLine start end))))
       ((if (state property-drawer-open)
            ((if (line-starts-with-ascii-ci ":END:")
                 ((token DrawerEndLine start end) (finish-node)
                  (set-bool property-drawer-open (bool #f)))
                 ((if (line-has-key-after-prefix? ":")
                      ((start-node OrgNodeProperty)
                       (token PropertyTrivia start (line-prefix-end ":"))
                       (token PropertyKey (line-prefix-end ":")
                              (line-scan-key (line-prefix-end ":")))
                       (token PropertyTrivia
                              (line-scan-key (line-prefix-end ":"))
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key (line-prefix-end ":")))))
                       (token PropertyValue
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key (line-prefix-end ":"))))
                              (line-trim-end))
                       (token PropertyTrivia (line-trim-end) end)
                       (finish-node))
                      ((token TextLine start end))))))
            ((if (line-starts-with-ascii-ci ":PROPERTIES:")
                 ((if (state paragraph-open)
                      ((finish-node)
                       (set-bool paragraph-open (bool #f))) ())
                  (start-node OrgPropertyDrawer)
                  (token DrawerBeginLine start end)
                  (set-bool property-drawer-open (bool #t)))
                 ((if (line-starts-with-ascii-ci "#+begin_src")
            ((if (state paragraph-open)
                 ((finish-node) (set-bool paragraph-open (bool #f))) ())
             (start-node OrgSourceBlock)
             (token BlockBeginLine start (line-prefix-end "#+begin_src"))
             (if (line-has-word-after-prefix? "#+begin_src")
                 ((token BlockHeaderTrivia
                         (line-prefix-end "#+begin_src")
                         (line-skip-horizontal
                          (line-prefix-end "#+begin_src")))
                  (token SourceLanguage
                         (line-skip-horizontal
                          (line-prefix-end "#+begin_src"))
                         (line-scan-word
                          (line-skip-horizontal
                           (line-prefix-end "#+begin_src"))))
                  (token BlockHeaderTrivia
                         (line-scan-word
                          (line-skip-horizontal
                           (line-prefix-end "#+begin_src"))) end))
                 ((token BlockHeaderTrivia
                         (line-prefix-end "#+begin_src") end)))
             (set-bool source-block-open (bool #t)))
            ((if (uint-positive? (line-marker-level "*" " "))
                 ((if (state paragraph-open)
                      ((finish-node) (set-bool paragraph-open (bool #f))) ())
                  (close-through open-levels (line-marker-level "*" " "))
                  (open-level open-levels (line-marker-level "*" " ")
                              OrgSection)
                  (start-node OrgHeadline)
                  (token HeadlineLine start (line-marker-end "*" " "))
                  (token HeadlineTrivia
                         (line-marker-end "*" " ")
                         (line-skip-horizontal
                          (line-marker-end "*" " ")))
                  (token HeadlineTitle
                         (line-skip-horizontal
                          (line-marker-end "*" " "))
                         (line-trim-end))
                  (token HeadlineTrivia (line-trim-end) end)
                  (finish-node))
                 ((if (line-has-key-after-prefix? "#+")
                      ((if (state paragraph-open)
                           ((finish-node)
                            (set-bool paragraph-open (bool #f))) ())
                       (if (line-starts-with-ascii-ci "#+CALL:")
                           ((start-node OrgBabelCall))
                           ((start-node OrgKeyword)))
                       (token KeywordTrivia start (line-prefix-end "#+"))
                       (token KeywordKey (line-prefix-end "#+")
                              (line-scan-key (line-prefix-end "#+")))
                       (token KeywordTrivia
                              (line-scan-key (line-prefix-end "#+"))
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key (line-prefix-end "#+")))))
                       (token KeywordValue
                              (line-skip-horizontal
                               (line-step
                                (line-scan-key (line-prefix-end "#+"))))
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
                            (token TextLine start end) (finish-node))))))))))))))))
  ((if (state source-block-open) ((finish-node)) ())
   (if (state property-drawer-open) ((finish-node)) ())
   (if (state paragraph-open) ((finish-node)) ())
   (close-all open-levels)))
