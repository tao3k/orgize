;;; -*- Gerbil -*-
;;; Parser-specific AST checks preserve nesting and source byte spans.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in :std/misc/ports read-all-as-string)
        (only-in "modules/org-parser/test-syntax.ss"
                 check-org-ast org-events-cover-source?)
        (only-in "outline-event-fixture.ss" outline-event-fixture-json))
(export org-v1-outline-events-test)

(def org-v1-outline-events-test
  (test-suite "Org Scheme structural AST"
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
    (test-case "Rowan fixture matches executable Scheme events"
      (let* ((options (JSONReadOptions object-as-hash: #t))
             (generated (string->json (outline-event-fixture-json) options))
             (saved (call-with-input-file
                     "languages/org/v1/generated/outline-event-fixture.json"
                     (lambda (port)
                       (string->json (read-all-as-string port) options)))))
        (check (hash-get saved "source") => (hash-get generated "source"))
        (check (hash-get saved "events") => (hash-get generated "events"))))))
