;;; -*- Gerbil -*-
;;; Org-owned contextual event algorithm compiled by the generic Rowan AOT path.
;;; This replaces the flat line-event fixture; full Element cutover is pending.

(import (only-in :gerbil-parser/rust-rowan-event-support
                 define-event-fold-parser)
        (only-in "grammar.ss" org-v1-language-grammar))
(export parse-org-rowan-events parse_org_rowan_events)

(define-event-fold-parser
  parse-org-rowan-events parse_org_rowan_events org-v1-language-grammar OrgFile
  source ((open-levels (uint-stack)) (source-block-open #f))
  ((if (state source-block-open)
       ((if (line-starts-with-ascii-ci "#+end_src")
            ((token BlockEndLine start end) (finish-node)
             (set-bool source-block-open (bool #f)))
            ((token TextLine start end))))
       ((if (line-starts-with-ascii-ci "#+begin_src")
            ((start-node OrgSourceBlock) (token BlockBeginLine start end)
             (set-bool source-block-open (bool #t)))
            ((if (uint-positive? (line-marker-level "*" " "))
                 ((close-through open-levels (line-marker-level "*" " "))
                  (open-level open-levels (line-marker-level "*" " ")
                              OrgSection)
                  (start-node OrgHeadline)
                  (token HeadlineLine start end) (finish-node))
                 ((start-node OrgTextLine)
                  (token TextLine start end) (finish-node))))))))
  ((if (state source-block-open) ((finish-node)) ())
   (close-all open-levels)))
