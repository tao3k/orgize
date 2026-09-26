;;; -*- Gerbil -*-
;;; Parser-specific AST checks preserve nesting and source byte spans.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in :std/misc/ports read-all-as-string)
        (only-in "modules/org-parser/test-syntax.ss"
                 check-org-ast org-events-cover-source?)
        (only-in "modules/org-parser/funs.ss" org-line-spans)
        (only-in "outline-event-fixture.ss" outline-event-fixture-json))
(export org-v1-outline-events-test)

(def org-v1-outline-events-test
  (test-suite "Org Scheme structural AST"
    (test-case "line spans follow UTF-8 bytes, CRLF, bare CR and EOF"
      (check (org-line-spans (string->utf8 "α\r\nb\rc\nlast"))
             => '((0 . 4) (4 . 6) (6 . 8) (8 . 12)))
      (check (org-line-spans (string->utf8 "")) => '())
      (check (org-line-spans (string->utf8 "x\n")) => '((0 . 2))))
    (test-case "mixed line endings preserve paragraph byte coverage"
      (check-org-ast "α\r\nb\rc\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 4))
          (OrgTextLine (TextLine 4 6))
          (OrgTextLine (TextLine 6 8))))))
    (test-case "inline links preserve target, description and malformed text"
      (check-org-ast "go [[https://a][α]] and [[id:b]]\n[[broken\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 3)
           (OrgLink (LinkTrivia 3 5) (LinkTarget 5 14)
                    (LinkTrivia 14 16) (LinkDescription 16 18)
                    (LinkTrivia 18 20))
           (TextLine 20 25)
           (OrgLink (LinkTrivia 25 27) (LinkTarget 27 31)
                    (LinkTrivia 31 33))
           (TextLine 33 34))
          (OrgTextLine (TextLine 34 43))))))
    (test-case "nested and sibling sections"
      (check-org-ast "* Parent\n** Child\ntext\n* Peer\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 8) (HeadlineTrivia 8 9))
          (OrgSection
           (OrgHeadline (HeadlineLine 9 11) (HeadlineTrivia 11 12)
                        (HeadlineTitle 12 17) (HeadlineTrivia 17 18))
           (OrgParagraph (OrgTextLine (TextLine 18 23)))))
         (OrgSection
          (OrgHeadline (HeadlineLine 23 24) (HeadlineTrivia 24 25)
                       (HeadlineTitle 25 29) (HeadlineTrivia 29 30))))))
    (test-case "non-headline stars and UTF-8 byte spans"
      (check-org-ast "*not a headline\n* α\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 16)))
         (OrgSection
          (OrgHeadline (HeadlineLine 16 17) (HeadlineTrivia 17 18)
                       (HeadlineTitle 18 20) (HeadlineTrivia 20 21))))))
    (test-case "empty document"
      (check-org-ast "" (OrgFile)))
    (test-case "AST assertion rejects missing and overlapping source spans"
      (check (org-events-cover-source?
              "abc" '((start OrgFile) (token TextLine 0 2) (finish)))
             => #f)
      (check (org-events-cover-source?
              "abc" '((start OrgFile) (token TextLine 0 2)
                      (token TextLine 1 3) (finish)))
             => #f))
    (test-case "blank lines separate paragraphs"
      (check-org-ast "alpha\nbeta\n\nnext\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 6))
                       (OrgTextLine (TextLine 6 11)))
         (OrgTextLine (TextLine 11 12))
         (OrgParagraph (OrgTextLine (TextLine 12 17))))))
    (test-case "nested and sibling lists are Scheme-owned Elements"
      (check-org-ast "- a\n  - b\n- c\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (OrgParagraph (OrgTextLine (TextLine 2 4)))
           (OrgPlainList
            (OrgListItem
             (ListTrivia 4 6) (ListBullet 6 7) (ListTrivia 7 8)
             (OrgParagraph (OrgTextLine (TextLine 8 10))))))
          (OrgListItem
           (ListBullet 10 11) (ListTrivia 11 12)
           (OrgParagraph (OrgTextLine (TextLine 12 14)))))))
      (check-org-ast "1. a\n2) b\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2) (ListTrivia 2 3)
           (OrgParagraph (OrgTextLine (TextLine 3 5))))
          (OrgListItem
           (ListBullet 5 7) (ListTrivia 7 8)
           (OrgParagraph (OrgTextLine (TextLine 8 10)))))))
      (check-org-ast "\t- α\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListTrivia 0 1) (ListBullet 1 2) (ListTrivia 2 3)
           (OrgParagraph (OrgTextLine (TextLine 3 6))))))))
    (test-case "list continuation and blank separation preserve item scope"
      (check-org-ast "- alpha\n  more\n- beta\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (OrgParagraph
            (OrgTextLine (TextLine 2 8))
            (OrgTextLine (TextLine 8 15))))
          (OrgListItem
           (ListBullet 15 16) (ListTrivia 16 17)
           (OrgParagraph (OrgTextLine (TextLine 17 22)))))))
      (check-org-ast "- a\n\n- b\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (OrgParagraph (OrgTextLine (TextLine 2 4)))
           (ListTrivia 4 5))
          (OrgListItem
           (ListBullet 5 6) (ListTrivia 6 7)
           (OrgParagraph (OrgTextLine (TextLine 7 9))))))))
    (test-case "element blocks recursively parse paragraphs and lists"
      (check-org-ast "#+begin_quote\ntext\n- item\n#+end_quote\n"
        (OrgFile
         (OrgQuoteBlock
          (BlockBeginLine 0 14)
          (OrgParagraph (OrgTextLine (TextLine 14 19)))
          (OrgPlainList
           (OrgListItem
            (ListBullet 19 20) (ListTrivia 20 21)
            (OrgParagraph (OrgTextLine (TextLine 21 26)))))
          (BlockEndLine 26 38))))
      (check-org-ast
       "#+begin_center\n#+begin_quote\nα\n#+end_quote\n#+end_center\n"
        (OrgFile
         (OrgCenterBlock
          (BlockBeginLine 0 15)
          (OrgQuoteBlock
           (BlockBeginLine 15 29)
           (OrgParagraph (OrgTextLine (TextLine 29 32)))
           (BlockEndLine 32 44))
          (BlockEndLine 44 57)))))
    (test-case "named dynamic blocks and drawers keep typed headers"
      (check-org-ast "#+BEGIN: note\ntext\n#+END:\n"
        (OrgFile
         (OrgDynamicBlock
          (BlockBeginLine 0 8) (DynamicBlockHeaderTrivia 8 9)
          (DynamicBlockName 9 13) (DynamicBlockHeaderTrivia 13 14)
          (OrgParagraph (OrgTextLine (TextLine 14 19)))
          (BlockEndLine 19 26))))
      (check-org-ast ":LOGBOOK:\nentry\n:END:\n"
        (OrgFile
         (OrgDrawer
          (DrawerBeginLine 0 1) (DrawerName 1 8)
          (DrawerTrivia 8 10)
          (OrgParagraph (OrgTextLine (TextLine 10 16)))
          (DrawerEndLine 16 22)))))
    (test-case "closing delimiters and invalid dynamic names are not openings"
      (check-org-ast ":END:\ntext\n:END:\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 6))
          (OrgTextLine (TextLine 6 11))
          (OrgTextLine (TextLine 11 17)))))
      (check-org-ast "#+BEGIN: 123\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 7)
          (KeywordTrivia 7 9) (KeywordValue 9 12)
          (KeywordTrivia 12 13)))))
    (test-case "block closer requires only trailing whitespace"
      (check-org-ast
       "#+begin_quote\ntext\n#+end_quote junk\n* Next\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 14))
          (OrgTextLine (TextLine 14 19))
          (OrgTextLine (TextLine 19 36)))
         (OrgSection
          (OrgHeadline
           (HeadlineLine 36 37) (HeadlineTrivia 37 38)
           (HeadlineTitle 38 42) (HeadlineTrivia 42 43))))))
    (test-case "table rows, rule row, escaped bar and UTF-8 cells"
      (check-org-ast "| a | b |\n|---+---|\n| c\\|d | α |\n\n* Next\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1) (OrgTableCell (TableCellText 1 4))
           (TableSeparator 4 5) (OrgTableCell (TableCellText 5 8))
           (TableSeparator 8 9) (TableTrivia 9 10))
          (OrgTableRuleRow (TableRuleText 10 20))
          (OrgTableRow
           (TableSeparator 20 21) (OrgTableCell (TableCellText 21 27))
           (TableSeparator 27 28) (OrgTableCell (TableCellText 28 32))
           (TableSeparator 32 33) (TableTrivia 33 34)))
         (OrgTextLine (TextLine 34 35))
         (OrgSection
          (OrgHeadline (HeadlineLine 35 36) (HeadlineTrivia 36 37)
                       (HeadlineTitle 37 41) (HeadlineTrivia 41 42))))))
    (test-case "indent, missing trailing bar and table boundaries"
      (check-org-ast "  | x\ntext\n|z|\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableTrivia 0 2) (TableSeparator 2 3)
           (OrgTableCell (TableCellText 3 5)) (TableTrivia 5 6)))
         (OrgParagraph (OrgTextLine (TextLine 6 11)))
         (OrgTable
          (OrgTableRow
           (TableSeparator 11 12) (OrgTableCell (TableCellText 12 13))
           (TableSeparator 13 14) (TableTrivia 14 15))))))
    (test-case "Babel CALL retains its own Element kind"
      (check-org-ast "#+CALL: build(input=42)\n"
        (OrgFile
         (OrgBabelCall
          (KeywordTrivia 0 2) (KeywordKey 2 6)
          (KeywordTrivia 6 8) (KeywordValue 8 23)
          (KeywordTrivia 23 24)))))
    (test-case "Planning is headline-local while CLOCK is a keyed Element"
      (check-org-ast
       "SCHEDULED: outside\n* Task\nSCHEDULED: <2026-01-01>\nCLOCK: [a]--[b]\nSCHEDULED: later\n"
       (OrgFile
        (OrgParagraph (OrgTextLine (TextLine 0 19)))
        (OrgSection
         (OrgHeadline (HeadlineLine 19 20) (HeadlineTrivia 20 21)
                      (HeadlineTitle 21 25) (HeadlineTrivia 25 26))
         (OrgPlanning (PlanningKey 26 35) (PlanningTrivia 35 37)
                      (PlanningValue 37 49) (PlanningTrivia 49 50))
         (OrgClock (ClockKey 50 55) (ClockTrivia 55 57)
                   (ClockValue 57 65) (ClockTrivia 65 66))
         (OrgParagraph (OrgTextLine (TextLine 66 83)))))))
    (test-case "source block remains inside its section"
      (check-org-ast "* Code\n#+BEGIN_SRC rust\nα\n#+END_SRC\n** Next\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 6) (HeadlineTrivia 6 7))
          (OrgSourceBlock
           (BlockBeginLine 7 18) (BlockHeaderTrivia 18 19)
           (SourceLanguage 19 23) (BlockHeaderTrivia 23 24)
           (TextLine 24 27) (BlockEndLine 27 37))
          (OrgSection
           (OrgHeadline (HeadlineLine 37 39) (HeadlineTrivia 39 40)
                        (HeadlineTitle 40 44) (HeadlineTrivia 44 45)))))))
    (test-case "unclosed source block recovers before a headline"
      (check-org-ast "* First\n#+begin_src rust\nbody\n** Next\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 7) (HeadlineTrivia 7 8))
          (OrgParagraph
           (OrgTextLine (TextLine 8 25))
           (OrgTextLine (TextLine 25 30)))
          (OrgSection
           (OrgHeadline (HeadlineLine 30 32) (HeadlineTrivia 32 33)
                        (HeadlineTitle 33 37) (HeadlineTrivia 37 38)))))))
    (test-case "opaque block declarations keep their bodies literal"
      (check-org-ast
       "#+begin_example\n| not table |\n#+end_example\n#+begin_export html\n<b>α</b>\n#+end_export\n#+begin_comment\nignored\n#+end_comment\n"
       (OrgFile
        (OrgExampleBlock (BlockBeginLine 0 16) (TextLine 16 30)
                         (BlockEndLine 30 44))
        (OrgExportBlock (BlockBeginLine 44 58)
                        (BlockHeaderTrivia 58 59) (ExportBackend 59 63)
                        (BlockHeaderTrivia 63 64) (TextLine 64 74)
                        (BlockEndLine 74 87))
        (OrgCommentBlock (BlockBeginLine 87 103) (TextLine 103 111)
                         (BlockEndLine 111 125)))))
    (test-case "unclosed example recovers before the next headline"
      (check-org-ast "#+begin_example\nbody\n* Next\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 16))
                       (OrgTextLine (TextLine 16 21)))
         (OrgSection
          (OrgHeadline (HeadlineLine 21 22) (HeadlineTrivia 22 23)
                       (HeadlineTitle 23 27) (HeadlineTrivia 27 28))))))
    (test-case "property drawer emits typed keys, values and empty values"
      (check-org-ast "* Task\n:PROPERTIES:\n:ID: alpha\n:EMPTY:\n:END:\nbody\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 6) (HeadlineTrivia 6 7))
          (OrgPropertyDrawer
           (DrawerBeginLine 7 20)
           (OrgNodeProperty
            (PropertyTrivia 20 21) (PropertyKey 21 23)
            (PropertyTrivia 23 25) (PropertyValue 25 30)
            (PropertyTrivia 30 31))
           (OrgNodeProperty
            (PropertyTrivia 31 32) (PropertyKey 32 37)
            (PropertyTrivia 37 38) (PropertyTrivia 38 39))
           (DrawerEndLine 39 45))
          (OrgParagraph (OrgTextLine (TextLine 45 50))))))
    (test-case "invalid property body recovers as text"
      (check-org-ast ":PROPERTIES:\n:BAD:value\n:END:\n* Next\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 13))
          (OrgTextLine (TextLine 13 24))
          (OrgTextLine (TextLine 24 30)))
         (OrgSection
          (OrgHeadline (HeadlineLine 30 31) (HeadlineTrivia 31 32)
                       (HeadlineTitle 32 36) (HeadlineTrivia 36 37))))))
    (test-case "Rowan fixture matches executable Scheme events"
      (let* ((options (JSONReadOptions object-as-hash: #t))
             (generated (string->json (outline-event-fixture-json) options))
             (saved (call-with-input-file
                     "languages/org/v1/generated/outline-event-fixture.json"
                     (lambda (port)
                       (string->json (read-all-as-string port) options)))))
        (check (hash-get saved "source") => (hash-get generated "source"))
        (check (hash-get saved "events") => (hash-get generated "events")))))))
