;;; -*- Gerbil -*-
;;; The Org-owned algorithm executes as Scheme before AOT lowering.

(import (only-in :std/test check test-case test-suite)
        (only-in :clan/poo/object .o)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in :std/misc/ports read-all-as-string)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-blocks)
        (only-in "parser.ss" org-v1-line-structure)
        (only-in "modules/org-parser/types.ss"
                 org-event-block? org-named-block?
                 org-inline-markup? org-inline-script?
                 org-event-helper?
                 org-event-strategy?)
        (only-in "modules/org-parser/objects.ss"
                 make-org-event-block org-event-block-id
                 make-org-named-block
                 make-org-inline-markup org-inline-markup-node
                 make-org-inline-script org-inline-script-node
                 make-org-event-helper org-event-helper-descriptor
                 make-org-event-strategy org-event-strategy-root)
        (only-in "modules/org-parser/test-syntax.ss" check-org-ast-with)
        (only-in "rowan-event-fixture.ss" rowan-event-fixture-json)
        (only-in "rowan-event-parser.ss"
                 parse-org-rowan-events
                 parse-org-rowan-events-with-inlinetask-level
                 parse-org-rowan-events-with-inline-script-policy
                 parse_org_rowan_events))
(export org-v1-rowan-table-container-parser-test)

(def org-v1-rowan-table-container-parser-test
  (test-suite "Org tables containers and AOT receipts"
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
            (OrgTableCell (OrgTextLine (TextLine 5 8)))
            (TableSeparator 8 9)
            (OrgTableCell (OrgTextLine (TextLine 9 12)))
            (TableSeparator 12 13)
            (TableTrivia 13 14))
           (OrgTableRuleRow (TableRuleText 14 24)))
          (OrgParagraph (OrgTextLine (TextLine 24 30)))))))
    (test-case "TBLFM assignment structure is Scheme-owned inside its table"
      (check-org-ast-with parse-org-rowan-events
        "| a |\n#+TBLFM: $1=$2\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (OrgTextLine (TextLine 1 4)))
           (TableSeparator 4 5)
           (TableTrivia 5 6))
          (OrgKeyword
           (KeywordTrivia 6 8) (KeywordKey 8 13)
           (KeywordTrivia 13 14)
           (OrgKeywordRawValue
            (KeywordTrivia 14 15)
            (OrgTableFormulaValue
             (OrgTableFormulaAssignment
              (OrgTableFormulaLhs
               (OrgTableFormulaReference (FormulaFieldReference 15 17)))
              (FormulaEquals 17 18)
              (OrgTableFormulaRhs
               (OrgTableFormulaReference (FormulaFieldReference 18 20))))))
           (KeywordTrivia 20 21))))))
    (test-case "formula row ranges keep two source-backed references"
      (check-org-ast-with parse-org-rowan-events
        "| a |\n#+TBLFM: $2=@2..@4\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (OrgTextLine (TextLine 1 4)))
           (TableSeparator 4 5)
           (TableTrivia 5 6))
          (OrgKeyword
           (KeywordTrivia 6 8) (KeywordKey 8 13)
           (KeywordTrivia 13 14)
           (OrgKeywordRawValue
            (KeywordTrivia 14 15)
            (OrgTableFormulaValue
             (OrgTableFormulaAssignment
              (OrgTableFormulaLhs
               (OrgTableFormulaReference (FormulaFieldReference 15 17)))
              (FormulaEquals 17 18)
              (OrgTableFormulaRhs
               (OrgTableFormulaReference (FormulaRowReference 18 20))
               (FormulaText 20 22)
               (OrgTableFormulaReference (FormulaRowReference 22 24))))))
           (KeywordTrivia 24 25))))))
    (test-case "formula assignments and flags remain structured Scheme nodes"
      (check-org-ast-with parse-org-rowan-events
        "| a |\n#+TBLFM: $1=$2;N::$3=@2\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (OrgTextLine (TextLine 1 4)))
           (TableSeparator 4 5)
           (TableTrivia 5 6))
          (OrgKeyword
           (KeywordTrivia 6 8) (KeywordKey 8 13)
           (KeywordTrivia 13 14)
           (OrgKeywordRawValue
            (KeywordTrivia 14 15)
            (OrgTableFormulaValue
             (OrgTableFormulaAssignment
              (OrgTableFormulaLhs
               (OrgTableFormulaReference (FormulaFieldReference 15 17)))
              (FormulaEquals 17 18)
              (OrgTableFormulaRhs
               (OrgTableFormulaReference (FormulaFieldReference 18 20)))
              (FormulaFlagSeparator 20 21)
              (FormulaFlag 21 22))
             (FormulaSeparator 22 24)
             (OrgTableFormulaAssignment
              (OrgTableFormulaLhs
               (OrgTableFormulaReference (FormulaFieldReference 24 26)))
              (FormulaEquals 26 27)
              (OrgTableFormulaRhs
               (OrgTableFormulaReference (FormulaRowReference 27 29))))))
           (KeywordTrivia 29 30))))))
    (test-case "table.el border and cells remain one Scheme Element"
      (check-org-ast-with parse-org-rowan-events
        "  +---+\n  | a |\n  +---+\n"
        (OrgFile
         (OrgTableEl
          (TableElLine 0 8)
          (TableElLine 8 16)
          (TableElLine 16 24)))))
    (test-case "Scheme emits source-backed links inside table cells"
      (check-org-ast-with parse-org-rowan-events
        "| [[id:x]] |\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell
            (OrgTextLine
             (TextLine 1 2)
             (OrgLink (LinkTrivia 2 4) (LinkTarget 4 8)
                      (LinkTrivia 8 10))
             (TextLine 10 11)))
           (TableSeparator 11 12)
           (TableTrivia 12 13))))))
    (test-case "table delimiter escaping follows preceding backslash parity"
      (check-org-ast-with parse-org-rowan-events
        "| a\\|b | c |\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (OrgTextLine (TextLine 1 7)))
           (TableSeparator 7 8)
           (OrgTableCell (OrgTextLine (TextLine 8 11)))
           (TableSeparator 11 12)
           (TableTrivia 12 13)))))
      (check-org-ast-with parse-org-rowan-events
        "| a\\\\|b | c |\n"
        (OrgFile
         (OrgTable
          (OrgTableRow
           (TableSeparator 0 1)
           (OrgTableCell (OrgTextLine (TextLine 1 5)))
           (TableSeparator 5 6)
           (OrgTableCell (OrgTextLine (TextLine 6 8)))
           (TableSeparator 8 9)
           (OrgTableCell (OrgTextLine (TextLine 9 12)))
           (TableSeparator 12 13)
           (TableTrivia 13 14))))))
    (test-case "POO list markers retain nested and sibling item scopes"
      (check-org-ast-with parse-org-rowan-events "- a\n  - b\n- c\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2)
           (OrgParagraph (OrgTextLine (TextLine 2 4)))
           (OrgPlainList
            (OrgListItem
             (ListTrivia 4 6) (ListBullet 6 8)
             (OrgParagraph (OrgTextLine (TextLine 8 10))))))
          (OrgListItem
           (ListBullet 10 12)
           (OrgParagraph (OrgTextLine (TextLine 12 14)))))))
      (check-org-ast-with parse-org-rowan-events "1. a\n2) b\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 3)
           (OrgParagraph (OrgTextLine (TextLine 3 5))))
          (OrgListItem
           (ListBullet 5 8)
           (OrgParagraph (OrgTextLine (TextLine 8 10))))))))
    (test-case "list continuation and blank line remain within item"
      (check-org-ast-with parse-org-rowan-events "- alpha\n  more\n- beta\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2)
           (OrgParagraph
            (OrgTextLine (TextLine 2 15))))
          (OrgListItem
           (ListBullet 15 17)
           (OrgParagraph (OrgTextLine (TextLine 17 22)))))))
      (check-org-ast-with parse-org-rowan-events "- a\n\n- b\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2)
           (OrgParagraph (OrgTextLine (TextLine 2 4)))
           (ListTrivia 4 5))
          (OrgListItem
           (ListBullet 5 7)
           (OrgParagraph (OrgTextLine (TextLine 7 9))))))))
    (test-case "Scheme list algorithm emits typed counter checkbox and tag spans"
      (check-org-ast-with parse-org-rowan-events "- [@2] [X] done\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2)
           (ListTrivia 2 4) (ListCounterValue 4 5) (ListTrivia 5 7)
           (ListTrivia 7 8) (ListCheckboxValue 8 9) (ListTrivia 9 11)
           (OrgParagraph (OrgTextLine (TextLine 11 16)))))))
      (check-org-ast-with parse-org-rowan-events "- term :: body\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2)
           (ListTagValue 2 7) (ListTrivia 7 10)
           (OrgParagraph (OrgTextLine (TextLine 10 15))))))))
    (test-case "block and drawer markers do not consume longer lookalikes"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust\n#+end_srcx\n#+END_SRC \t\n"
        (OrgFile
         (OrgSourceBlock (BlockBeginLine 0 11)
                         (BlockHeaderTrivia 11 12)
                         (SourceLanguage 12 16)
                         (SourceHeaderTrivia 16 17)
                         (OrgBlockBodyLine (TextLine 17 28))
                         (BlockEndLine 28 40))))
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
         (OrgExampleBlock (BlockBeginLine 0 15)
                          (SourceHeaderTrivia 15 16)
                          (OrgBlockBodyLine (TextLine 16 25))
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
    (test-case "recursive containers keep nested Scheme-owned Elements"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_quote\ntext\n- item\n#+end_quote\n"
        (OrgFile
         (OrgQuoteBlock
          (BlockBeginLine 0 14)
          (OrgParagraph (OrgTextLine (TextLine 14 19)))
          (OrgPlainList
           (OrgListItem
            (ListBullet 19 21)
            (OrgParagraph (OrgTextLine (TextLine 21 26)))))
          (BlockEndLine 26 38))))
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN: note\ntext\n#+END:\n:LOGBOOK:\nentry\n:END:\n"
        (OrgFile
         (OrgDynamicBlock
          (BlockBeginLine 0 8) (DynamicBlockHeaderTrivia 8 9)
          (DynamicBlockName 9 13) (DynamicBlockHeaderTrivia 13 14)
          (OrgParagraph (OrgTextLine (TextLine 14 19)))
          (BlockEndLine 19 26))
         (OrgDrawer
          (DrawerBeginLine 26 27) (DrawerName 27 34)
          (DrawerTrivia 34 36)
          (OrgParagraph (OrgTextLine (TextLine 36 42)))
          (DrawerEndLine 42 48)))))
    (test-case "source-named special blocks match their closing names"
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_NOTE\ntext\n#+END_note\n"
        (OrgFile
         (OrgSpecialBlock
          (BlockBeginLine 0 8) (SpecialBlockName 8 12)
          (BlockHeaderTrivia 12 13)
          (OrgParagraph (OrgTextLine (TextLine 13 18)))
          (BlockEndLine 18 29))))
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_NOTE\n* heading\n#+END_note\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 13)))
         (OrgSection
          (OrgHeadline (HeadlineLine 13 14) (HeadlineTrivia 14 15)
                       (OrgHeadlineTitle (OrgTextLine (TextLine 15 22)))
                       (HeadlineTrivia 22 23))
          (OrgParagraph
           (OrgTextLine
            (TextLine 23 28)
            (OrgSubscript (InlineScriptDelimiter 28 29)
                          (InlineScriptValue 29 33))
            (TextLine 33 34))))))
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_NOTE\ntext\n#+END_OTHER\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 30))))))
    (test-case "nested named special blocks close one frame at a time"
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_OUTER\n#+BEGIN_INNER\nx\n#+END_inner\n#+END_outer\n"
        (OrgFile
         (OrgSpecialBlock
          (BlockBeginLine 0 8) (SpecialBlockName 8 13)
          (BlockHeaderTrivia 13 14)
          (OrgSpecialBlock
           (BlockBeginLine 14 22) (SpecialBlockName 22 27)
           (BlockHeaderTrivia 27 28)
           (OrgParagraph (OrgTextLine (TextLine 28 30)))
           (BlockEndLine 30 42))
          (BlockEndLine 42 54)))))
    (test-case "named child blocks do not cross a parent closer"
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_OUTER\n#+BEGIN_INNER\n#+END_outer\n#+END_inner\n"
        (OrgFile
         (OrgSpecialBlock
          (BlockBeginLine 0 8) (SpecialBlockName 8 13)
          (BlockHeaderTrivia 13 14)
          (OrgParagraph (OrgTextLine (TextLine 14 28)))
          (BlockEndLine 28 40))
         (OrgParagraph
          (OrgTextLine
           (TextLine 40 45)
           (OrgSubscript (InlineScriptDelimiter 45 46)
                         (InlineScriptValue 46 51))
           (TextLine 51 52)))))
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_NOTE\n\\begin{a}\n#+END_NOTE\n\\end{a}\n"
        (OrgFile
         (OrgSpecialBlock
          (BlockBeginLine 0 8) (SpecialBlockName 8 12)
          (BlockHeaderTrivia 12 13)
          (OrgParagraph (OrgTextLine (TextLine 13 23)))
          (BlockEndLine 23 34))
         (OrgParagraph (OrgTextLine (TextLine 34 42))))))
    (test-case "unrelated named closers do not truncate a child block"
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN_OUTER\n#+BEGIN_INNER\n#+END_other\n#+END_inner\n#+END_outer\n"
        (OrgFile
         (OrgSpecialBlock
          (BlockBeginLine 0 8) (SpecialBlockName 8 13)
          (BlockHeaderTrivia 13 14)
          (OrgSpecialBlock
           (BlockBeginLine 14 22) (SpecialBlockName 22 27)
           (BlockHeaderTrivia 27 28)
           (OrgParagraph
            (OrgTextLine
             (TextLine 28 33)
             (OrgSubscript (InlineScriptDelimiter 33 34)
                           (InlineScriptValue 34 39))
             (TextLine 39 40)))
           (BlockEndLine 40 52))
          (BlockEndLine 52 64)))))
    (test-case "source-named LaTeX environments keep opaque bodies"
      (check-org-ast-with parse-org-rowan-events
        "\\begin{align*}\nx\n\\end{align*}\n"
        (OrgFile
         (OrgLatexEnvironment
          (LatexEnvironmentBegin 0 7)
          (LatexEnvironmentName 7 13)
          (LatexEnvironmentBeginSuffix 13 14)
          (LatexEnvironmentBody 14 15)
          (LatexEnvironmentBody 15 17)
          (LatexEnvironmentEnd 17 30))))
      (check-org-ast-with parse-org-rowan-events
        "\\begin{a}\\end{a}"
        (OrgFile
         (OrgLatexEnvironment
          (LatexEnvironmentBegin 0 7)
          (LatexEnvironmentName 7 8)
          (LatexEnvironmentBeginSuffix 8 9)
          (LatexEnvironmentEnd 9 16))))
      (check-org-ast-with parse-org-rowan-events
        "\\begin{a}x\\foo \\end{a}"
        (OrgFile
         (OrgLatexEnvironment
          (LatexEnvironmentBegin 0 7)
          (LatexEnvironmentName 7 8)
          (LatexEnvironmentBeginSuffix 8 9)
          (LatexEnvironmentBody 9 15)
          (LatexEnvironmentEnd 15 22))))
      (check-org-ast-with parse-org-rowan-events
        "\\begin{a}\n* heading\n\\end{a}\n"
        (OrgFile
         (OrgLatexEnvironment
          (LatexEnvironmentBegin 0 7)
          (LatexEnvironmentName 7 8)
          (LatexEnvironmentBeginSuffix 8 9)
          (LatexEnvironmentBody 9 10)
          (LatexEnvironmentBody 10 20)
          (LatexEnvironmentEnd 20 28)))))
    (test-case "LaTeX environment markers and names are ASCII case-insensitive"
      (check-org-ast-with parse-org-rowan-events
        "\\BEGIN{AlIgN*}\nx\n\\EnD{aLiGn*}\n"
        (OrgFile
         (OrgLatexEnvironment
          (LatexEnvironmentBegin 0 7)
          (LatexEnvironmentName 7 13)
          (LatexEnvironmentBeginSuffix 13 14)
          (LatexEnvironmentBody 14 15)
          (LatexEnvironmentBody 15 17)
          (LatexEnvironmentEnd 17 30))))
      (check-org-ast-with parse-org-rowan-events
        "\\BEGIN{A}\\eNd{a}"
        (OrgFile
         (OrgLatexEnvironment
          (LatexEnvironmentBegin 0 7)
          (LatexEnvironmentName 7 8)
          (LatexEnvironmentBeginSuffix 8 9)
          (LatexEnvironmentEnd 9 16)))))
    (test-case "indented container delimiters and orphan closers retain source"
      (check-org-ast-with parse-org-rowan-events
        "  #+begin_quote\nx\n  #+end_quote\n"
        (OrgFile
         (OrgQuoteBlock
          (BlockBeginLine 0 16)
          (OrgParagraph (OrgTextLine (TextLine 16 18)))
          (BlockEndLine 18 32))))
      (check-org-ast-with parse-org-rowan-events
        " :LOGBOOK:\nentry\n :END:\n"
        (OrgFile
         (OrgDrawer
          (DrawerBeginLine 0 2) (DrawerName 2 9)
          (DrawerTrivia 9 11)
          (OrgParagraph (OrgTextLine (TextLine 11 17)))
          (DrawerEndLine 17 24))))
      (check-org-ast-with parse-org-rowan-events
        ":END:\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 6)))))
      (check-org-ast-with parse-org-rowan-events
        "  #+BEGIN: note\nx\n  #+END:\n"
        (OrgFile
         (OrgDynamicBlock
          (BlockBeginLine 0 10) (DynamicBlockHeaderTrivia 10 11)
          (DynamicBlockName 11 15) (DynamicBlockHeaderTrivia 15 16)
          (OrgParagraph (OrgTextLine (TextLine 16 18)))
          (BlockEndLine 18 27))))
      (check-org-ast-with parse-org-rowan-events
        "#+begin_center\n#+begin_quote\nα\n#+end_quote\n#+end_center\n"
        (OrgFile
         (OrgCenterBlock
          (BlockBeginLine 0 15)
          (OrgQuoteBlock
           (BlockBeginLine 15 29)
           (OrgParagraph (OrgTextLine (TextLine 29 32)))
           (BlockEndLine 32 44))
          (BlockEndLine 44 57)))))
    (test-case "Rowan fixture is projected by the same Scheme event algorithm"
      (let* ((options (JSONReadOptions object-as-hash: #t))
             (generated (string->json (rowan-event-fixture-json) options))
             (saved (call-with-input-file
                     "languages/org/v1/generated/rowan-event-fixture.json"
                     (lambda (port)
                       (string->json (read-all-as-string port) options)))))
        (check (hash-get saved "source") => (hash-get generated "source"))
        (check (hash-get saved "events") => (hash-get generated "events"))))
    (test-case "AOT IR is a typed source-owned event function"
      (let (ir (string->json parse_org_rowan_events
                             (JSONReadOptions object-as-hash: #t
                                              array-as-vector: #t)))
        (check (hash-ref ir "schema")
               => "gerbil-scheme-rust.event-function-ir.v1")
        (check (hash-ref ir "name") => "parse_org_rowan_events")
        (check (vector-length (hash-ref ir "line")) => 2)))))
