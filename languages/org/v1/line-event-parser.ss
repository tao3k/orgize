;;; -*- Gerbil -*-
;;; Org-owned first event algorithm slice: flat headline/text lines only.
;;; Contextual nesting and blocks are not admitted by this function.

(import (only-in :gerbil-parser/src/compiler/event-strategy-aot
                 define-line-event-parser event-node event-token
                 line-starts-with?)
        (only-in "grammar.ss" org-v1-language-grammar))
(export parse-org-line-events parse_org_line_events)

(define-line-event-parser
  parse-org-line-events parse_org_line_events org-v1-language-grammar OrgFile
  source (line start end)
  (if (line-starts-with? line "* ")
    (event-node OrgHeadline (event-token HeadlineLine start end))
    (event-node OrgTextLine (event-token TextLine start end))))
