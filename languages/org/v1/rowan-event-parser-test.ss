;;; -*- Gerbil -*-
;;; The Org-owned algorithm executes as Scheme before AOT lowering.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in "modules/org-parser/test-syntax.ss" check-org-ast-with)
        (only-in "rowan-event-parser.ss"
                 parse-org-rowan-events parse_org_rowan_events))
(export org-v1-rowan-event-parser-test)

(def org-v1-rowan-event-parser-test
  (test-suite "Org contextual Rowan event AOT"
    (test-case "paragraphs group source lines and blank trivia closes the scope"
      (check-org-ast-with parse-org-rowan-events
        "alpha\nβ\n \t\nnext\n* H\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 6))
                       (OrgTextLine (TextLine 6 9)))
         (OrgTextLine (TextLine 9 12))
         (OrgParagraph (OrgTextLine (TextLine 12 17)))
         (OrgSection (OrgHeadline (HeadlineLine 17 21))))))
    (test-case "source blocks mask headline syntax and sections retain nesting"
      (check-org-ast-with parse-org-rowan-events
        "* Parent\n#+BeGiN_SrC rust\n** fake\n#+EnD_SrC\n** Child\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 9))
          (OrgSourceBlock (BlockBeginLine 9 20)
                          (BlockHeaderTrivia 20 21)
                          (SourceLanguage 21 25)
                          (BlockHeaderTrivia 25 26)
                          (TextLine 26 34) (BlockEndLine 34 44))
          (OrgSection (OrgHeadline (HeadlineLine 44 53)))))))
    (test-case "unterminated blocks close at EOF without treating body as headings"
      (check-org-ast-with parse-org-rowan-events
        "* Parent\n#+BEGIN_SRC\n** body\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 9))
          (OrgSourceBlock (BlockBeginLine 9 20)
                          (BlockHeaderTrivia 20 21)
                          (TextLine 21 29))))))
    (test-case "AOT IR is a typed source-owned event function"
      (let (ir (string->json parse_org_rowan_events
                             (JSONReadOptions object-as-hash: #t
                                              array-as-vector: #t)))
        (check (hash-ref ir "schema")
               => "gerbil-scheme-rust.event-function-ir.v1")
        (check (hash-ref ir "name") => "parse_org_rowan_events")))))
