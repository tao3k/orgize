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
         (OrgSection (OrgHeadline (HeadlineLine 17 18)
                                  (HeadlineTrivia 18 19)
                                  (HeadlineTitle 19 20)
                                  (HeadlineTrivia 20 21))))))
    (test-case "source blocks mask headline syntax and sections retain nesting"
      (check-org-ast-with parse-org-rowan-events
        "* Parent\n#+BeGiN_SrC rust\n** fake\n#+EnD_SrC\n** Child\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 8) (HeadlineTrivia 8 9))
          (OrgSourceBlock (BlockBeginLine 9 20)
                          (BlockHeaderTrivia 20 21)
                          (SourceLanguage 21 25)
                          (BlockHeaderTrivia 25 26)
                          (TextLine 26 34) (BlockEndLine 34 44))
          (OrgSection (OrgHeadline (HeadlineLine 44 46)
                                    (HeadlineTrivia 46 47)
                                    (HeadlineTitle 47 52)
                                    (HeadlineTrivia 52 53)))))))
    (test-case "unterminated blocks close at EOF without treating body as headings"
      (check-org-ast-with parse-org-rowan-events
        "* Parent\n#+BEGIN_SRC\n** body\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 8) (HeadlineTrivia 8 9))
          (OrgSourceBlock (BlockBeginLine 9 20)
                          (BlockHeaderTrivia 20 21)
                          (TextLine 21 29))))))
    (test-case "file-local TODO and Babel CALL keys project as distinct Elements"
      (check-org-ast-with parse-org-rowan-events
        "#+SEQ_TODO: TODO | DONE \r\n* TODO Work\n#+CALL: name()\n"
        (OrgFile
         (OrgKeyword (KeywordTrivia 0 2) (KeywordKey 2 10)
                     (KeywordTrivia 10 12) (KeywordValue 12 23)
                     (KeywordTrivia 23 26))
         (OrgSection
          (OrgHeadline (HeadlineLine 26 27) (HeadlineTrivia 27 28)
                       (HeadlineTitle 28 37) (HeadlineTrivia 37 38))
          (OrgBabelCall (KeywordTrivia 38 40) (KeywordKey 40 44)
                        (KeywordTrivia 44 46) (KeywordValue 46 52)
                        (KeywordTrivia 52 53))))))
    (test-case "property drawer keys stay beneath the owning headline"
      (check-org-ast-with parse-org-rowan-events
        "* H\n:PROPERTIES:\n:ID: alpha\n:END:\nbody\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPropertyDrawer
           (DrawerBeginLine 4 17)
           (OrgNodeProperty (PropertyTrivia 17 18) (PropertyKey 18 20)
                            (PropertyTrivia 20 22) (PropertyValue 22 27)
                            (PropertyTrivia 27 28))
           (DrawerEndLine 28 34))
          (OrgParagraph (OrgTextLine (TextLine 34 39)))))))
    (test-case "declared planning and clock keys retain headline context"
      (check-org-ast-with parse-org-rowan-events
        "* H\nSCHEDULED: now\nCLOCK: 2\n* N\nDEADLINE: x\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPlanning (PlanningKey 4 13) (PlanningTrivia 13 15)
                       (PlanningValue 15 18) (PlanningTrivia 18 19))
          (OrgClock (ClockKey 19 24) (ClockTrivia 24 26)
                    (ClockValue 26 27) (ClockTrivia 27 28)))
         (OrgSection
          (OrgHeadline (HeadlineLine 28 29) (HeadlineTrivia 29 30)
                       (HeadlineTitle 30 31) (HeadlineTrivia 31 32))
          (OrgPlanning (PlanningKey 32 40) (PlanningTrivia 40 42)
                       (PlanningValue 42 43) (PlanningTrivia 43 44))))))
    (test-case "planning is not promoted after ordinary paragraph content"
      (check-org-ast-with parse-org-rowan-events
        "* H\nbody\nSCHEDULED: later\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgParagraph (OrgTextLine (TextLine 4 9))
                        (OrgTextLine (TextLine 9 26)))))))
    (test-case "empty declared values keep source spans ordered"
      (check-org-ast-with parse-org-rowan-events
        "* H\nSCHEDULED:  \n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPlanning (PlanningKey 4 13) (PlanningTrivia 13 16)
                       (PlanningTrivia 16 17))))))
    (test-case "POO table rows and rule rows AOT-fold into one table Element"
      (check-org-ast-with parse-org-rowan-events
        "* H\n| a | b |\n|---+---|\nplain\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgTable
           (OrgTableRow
            (TableSeparator 4 5)
            (OrgTableCell (TableCellText 5 8))
            (TableSeparator 8 9)
            (OrgTableCell (TableCellText 9 12))
            (TableSeparator 12 13)
            (TableTrivia 13 14))
           (OrgTableRuleRow (TableRuleText 14 24)))
          (OrgParagraph (OrgTextLine (TextLine 24 30)))))))
    (test-case "table delimiter escaping follows preceding backslash parity"
      (check-org-ast-with parse-org-rowan-events
        "| a\\|b | c |\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (TableCellText 1 7))
           (TableSeparator 7 8)
           (OrgTableCell (TableCellText 8 11))
           (TableSeparator 11 12)
           (TableTrivia 12 13)))))
      (check-org-ast-with parse-org-rowan-events
        "| a\\\\|b | c |\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (TableCellText 1 5))
           (TableSeparator 5 6)
           (OrgTableCell (TableCellText 6 8))
           (TableSeparator 8 9)
           (OrgTableCell (TableCellText 9 12))
           (TableSeparator 12 13)
           (TableTrivia 13 14))))))
    (test-case "block and drawer markers do not consume longer lookalikes"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust\n#+end_srcx\n#+END_SRC \t\n"
        (OrgFile
         (OrgSourceBlock (BlockBeginLine 0 11)
                         (BlockHeaderTrivia 11 12)
                         (SourceLanguage 12 16)
                         (BlockHeaderTrivia 16 17)
                         (TextLine 17 28) (BlockEndLine 28 40))))
      (check-org-ast-with parse-org-rowan-events
        "* H\n:PROPERTIES:\n:END: tail\n:END:\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPropertyDrawer (DrawerBeginLine 4 17)
                             (OrgNodeProperty
                              (PropertyTrivia 17 18) (PropertyKey 18 21)
                              (PropertyTrivia 21 23) (PropertyValue 23 27)
                              (PropertyTrivia 27 28))
                             (DrawerEndLine 28 34))))))
    (test-case "POO-declared opaque blocks share one Scheme AOT strategy"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_example\n* hidden\n#+end_example\n#+begin_comment\n| x |\n#+end_comment\n#+begin_export html\n<b>x</b>\n#+end_export\n* Visible\n"
        (OrgFile
         (OrgExampleBlock (BlockBeginLine 0 16) (TextLine 16 25)
                          (BlockEndLine 25 39))
         (OrgCommentBlock (BlockBeginLine 39 55) (TextLine 55 61)
                          (BlockEndLine 61 75))
         (OrgExportBlock (BlockBeginLine 75 89)
                         (BlockHeaderTrivia 89 90) (ExportBackend 90 94)
                         (BlockHeaderTrivia 94 95) (TextLine 95 104)
                         (BlockEndLine 104 117))
         (OrgSection
          (OrgHeadline (HeadlineLine 117 118) (HeadlineTrivia 118 119)
                       (HeadlineTitle 119 126) (HeadlineTrivia 126 127))))))
    (test-case "AOT IR is a typed source-owned event function"
      (let (ir (string->json parse_org_rowan_events
                             (JSONReadOptions object-as-hash: #t
                                              array-as-vector: #t)))
        (check (hash-ref ir "schema")
               => "gerbil-scheme-rust.event-function-ir.v1")
        (check (hash-ref ir "name") => "parse_org_rowan_events")))))
