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
                 org-event-block? org-inline-markup?)
        (only-in "modules/org-parser/objects.ss"
                 make-org-event-block org-event-block-id
                 make-org-inline-markup org-inline-markup-node)
        (only-in "modules/org-parser/test-syntax.ss" check-org-ast-with)
        (only-in "rowan-event-fixture.ss" rowan-event-fixture-json)
        (only-in "rowan-event-parser.ss"
                 parse-org-rowan-events parse_org_rowan_events))
(export org-v1-rowan-event-parser-test)

(def org-v1-rowan-event-parser-test
  (test-suite "Org contextual Rowan event AOT"
    (test-case "POO strategy declarations reject untyped rule tuples"
      (let ((block (make-org-event-block
                    1 (car (line-structure-blocks org-v1-line-structure))))
            (markup (make-org-inline-markup 42 3 'OrgBold)))
        (check (org-event-block? block) => #t)
        (check (org-event-block-id block) => 1)
        (check (org-inline-markup? markup) => #t)
        (check (org-inline-markup-node markup) => 'OrgBold)
        (check (org-event-block? '(1 . block)) => #f)
        (check (org-inline-markup? '(42 3 OrgBold)) => #f)
        (check (org-inline-markup?
                (.o kind: 'org-inline-markup byte: 256 id: 3
                    node: 'OrgBold))
               => #f)))
    (test-case "paragraphs group source lines and blank trivia closes the scope"
      (check-org-ast-with parse-org-rowan-events
        "alpha\nβ\n \t\nnext\n* H\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 9))
                       (OrgTextLine (TextLine 9 12)))
         (OrgParagraph (OrgTextLine (TextLine 12 17)))
         (OrgSection (OrgHeadline (HeadlineLine 17 18)
                                  (HeadlineTrivia 18 19)
                                  (HeadlineTitle 19 20)
                                  (HeadlineTrivia 20 21))))))
    (test-case "headline tags are source-backed fields, not Rust title parsing"
      (check-org-ast-with parse-org-rowan-events
        "* TODO Plan :agent:plan:\n"
        (OrgFile
         (OrgSection
          (OrgHeadline
           (HeadlineLine 0 1) (HeadlineTrivia 1 2)
           (HeadlineTitle 2 12)
           (HeadlineTagTrivia 12 13) (HeadlineTagValue 13 18)
           (HeadlineTagTrivia 18 19) (HeadlineTagValue 19 23)
           (HeadlineTagTrivia 23 24) (HeadlineTrivia 24 25)))))
      (check-org-ast-with parse-org-rowan-events
        "* Plan :bad::\n"
        (OrgFile
         (OrgSection
          (OrgHeadline
           (HeadlineLine 0 1) (HeadlineTrivia 1 2)
           (HeadlineTitle 2 13) (HeadlineTrivia 13 14))))))
    (test-case "Scheme macro Objects retain named and argument spans"
      (check-org-ast-with parse-org-rowan-events
        "x {{{title}}} y\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgMacro (MacroDelimiter 2 5) (MacroName 5 10)
                     (MacroDelimiter 10 13))
           (TextLine 13 16)))))
      (check-org-ast-with parse-org-rowan-events
        "{{{issue(42)}}}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgMacro (MacroDelimiter 0 3) (MacroName 3 8)
                     (MacroDelimiter 8 9) (MacroArguments 9 11)
                     (MacroDelimiter 11 15))
           (TextLine 15 16)))))
      (check-org-ast-with parse-org-rowan-events
        "{{{9bad}}} {{{broken\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 21))))))
    (test-case "the complete Org entity catalog drives source-backed Objects"
      (check-org-ast-with parse-org-rowan-events
        "\\cent \\alpha{} \\frac12{}test\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgEntity (EntityDelimiter 0 1) (EntityName 1 5))
           (TextLine 5 6)
           (OrgEntity (EntityDelimiter 6 7) (EntityName 7 12)
                      (EntityPost 12 14))
           (TextLine 14 15)
           (OrgEntity (EntityDelimiter 15 16) (EntityName 16 22)
                      (EntityPost 22 24))
           (TextLine 24 29)))))
      (check-org-ast-with parse-org-rowan-events
        "\\_   x\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgEntity (EntityDelimiter 0 1) (EntityName 1 2)
                      (EntityPost 2 5))
           (TextLine 5 7)))))
      (check-org-ast-with parse-org-rowan-events
        "\\unknown \\centaur\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 18)))))
      (check-org-ast-with parse-org-rowan-events
        "\\alpha\\beta\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgEntity (EntityDelimiter 0 1) (EntityName 1 6))
           (OrgEntity (EntityDelimiter 6 7) (EntityName 7 11))
           (TextLine 11 12)))))
      (check-org-ast-with parse-org-rowan-events
        "\\alpha[[https://example.org]]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgEntity (EntityDelimiter 0 1) (EntityName 1 6))
           (OrgLink (LinkTrivia 6 8)
                    (LinkTarget 8 27)
                    (LinkTrivia 27 29))
           (TextLine 29 30))))))
    (test-case "five-dash horizontal rule interrupts a paragraph"
      (check-org-ast-with parse-org-rowan-events
        "before\n-----\nafter\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 7)))
         (OrgHorizontalRule (HorizontalRuleLine 7 13))
         (OrgParagraph (OrgTextLine (TextLine 13 19))))))
    (test-case "fixed-width lines form one Element and stop at prose"
      (check-org-ast-with parse-org-rowan-events
        "first\n: A\n:\n: B\nlast\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 6)))
         (OrgFixedWidth (FixedWidthLine 6 10)
                        (FixedWidthLine 10 12)
                        (FixedWidthLine 12 16))
         (OrgParagraph (OrgTextLine (TextLine 16 21))))))
    (test-case "POO-declared inline links preserve descriptions and malformed text"
      (check-org-ast-with parse-org-rowan-events
        "go [[https://a][α]] and [[id:b]]\n[[broken\n"
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
           (TextLine 33 43))))))
    (test-case "target and radio-target Objects retain source-backed value spans"
      (check-org-ast-with parse-org-rowan-events
        "a <<one two>> and <<<radio>>> z\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgTarget (InlineTargetDelimiter 2 4)
                      (InlineTargetValue 4 11)
                      (InlineTargetDelimiter 11 13))
           (TextLine 13 18)
           (OrgRadioTarget (InlineTargetDelimiter 18 21)
                           (InlineTargetValue 21 26)
                           (InlineTargetDelimiter 26 29))
           (TextLine 29 32)))))
      (check-org-ast-with parse-org-rowan-events
        "x << bad>> and <<bad >>\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 24)))))
      (check-org-ast-with parse-org-rowan-events
        "<<β>>\r\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTarget (InlineTargetDelimiter 0 2)
                      (InlineTargetValue 2 4)
                      (InlineTargetDelimiter 4 6))
           (TextLine 6 8))))))
    (test-case "export snippets project backend, value, and lossless delimiters"
      (check-org-ast-with parse-org-rowan-events
        "hi @@html:<b>x</b>@@ ok\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 3)
           (OrgExportSnippet
            (ExportSnippetDelimiter 3 5)
            (ExportSnippetBackend 5 9)
            (ExportSnippetDelimiter 9 10)
            (ExportSnippetValue 10 18)
           (ExportSnippetDelimiter 18 20))
           (TextLine 20 24)))))
      (check-org-ast-with parse-org-rowan-events
        "a @@-:@@ b\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgExportSnippet
            (ExportSnippetDelimiter 2 4)
            (ExportSnippetBackend 4 5)
            (ExportSnippetDelimiter 5 6)
            (ExportSnippetDelimiter 6 8))
           (TextLine 8 11)))))
      (check-org-ast-with parse-org-rowan-events
        "a @@:x@@ and @@h_t:x@@ and @@html:x@\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 37))))))
    (test-case "footnote references preserve label, inline definition, and balanced brackets"
      (check-org-ast-with parse-org-rowan-events
        "x [fn:n] y\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgFootnoteReference
            (FootnoteReferenceDelimiter 2 6)
            (FootnoteReferenceLabel 6 7)
            (FootnoteReferenceDelimiter 7 8))
           (TextLine 8 11)))))
      (check-org-ast-with parse-org-rowan-events
        "x [fn::a [b]] z\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgFootnoteReference
            (FootnoteReferenceDelimiter 2 6)
            (FootnoteReferenceDelimiter 6 7)
            (FootnoteReferenceDefinition 7 12)
            (FootnoteReferenceDelimiter 12 13))
           (TextLine 13 16)))))
      (check-org-ast-with parse-org-rowan-events
        "[fn:n:a [b]]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgFootnoteReference
            (FootnoteReferenceDelimiter 0 4)
            (FootnoteReferenceLabel 4 5)
            (FootnoteReferenceDelimiter 5 6)
            (FootnoteReferenceDefinition 6 11)
            (FootnoteReferenceDelimiter 11 12))
           (TextLine 12 13)))))
      (check-org-ast-with parse-org-rowan-events
        "x [fn:] [fn::] [fn:bad name] [fn:no-close\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 42))))))
    (test-case "inline source blocks preserve language and balanced body spans"
      (check-org-ast-with parse-org-rowan-events
        "x src_rust{a{b}c} y\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgInlineSourceBlock
            (InlineCodeDelimiter 2 6)
            (InlineSourceLanguage 6 10)
            (InlineCodeDelimiter 10 11)
            (InlineSourceBody 11 16)
            (InlineCodeDelimiter 16 17))
           (TextLine 17 20)))))
      (check-org-ast-with parse-org-rowan-events
        "src_go{}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgInlineSourceBlock
            (InlineCodeDelimiter 0 4)
            (InlineSourceLanguage 4 6)
            (InlineCodeDelimiter 6 7)
            (InlineCodeDelimiter 7 8))
           (TextLine 8 9))))))
    (test-case "inline Babel calls preserve optional headers and nested arguments"
      (check-org-ast-with parse-org-rowan-events
        "call_foo[x](a(b))[z]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgInlineBabelCall
            (InlineCodeDelimiter 0 5)
            (InlineBabelCallName 5 8)
            (InlineCodeDelimiter 8 9)
            (InlineBabelInsideHeader 9 10)
            (InlineCodeDelimiter 10 12)
            (InlineBabelArguments 12 16)
            (InlineCodeDelimiter 16 18)
            (InlineBabelEndHeader 18 19)
            (InlineCodeDelimiter 19 20))
           (TextLine 20 21)))))
      (check-org-ast-with parse-org-rowan-events
        "call_foo(1)\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgInlineBabelCall
            (InlineCodeDelimiter 0 5)
            (InlineBabelCallName 5 8)
            (InlineCodeDelimiter 8 9)
            (InlineBabelArguments 9 10)
            (InlineCodeDelimiter 10 11))
           (TextLine 11 12))))))
    (test-case "incomplete inline code stays literal text"
      (check-org-ast-with parse-org-rowan-events
        "src_rust{unterminated\ncall_foo[x](unterminated\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 47)))))
      (check-org-ast-with parse-org-rowan-events
        "prefixsrc_rust{a}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 18))))))
    (test-case "footnote definitions contain elements and stop at headings"
      (check-org-ast-with parse-org-rowan-events
        "[fn:n] body\n* H\n"
        (OrgFile
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 12))))
         (OrgSection
          (OrgHeadline (HeadlineLine 12 13) (HeadlineTrivia 13 14)
                       (HeadlineTitle 14 15) (HeadlineTrivia 15 16)))))
      (check-org-ast-with parse-org-rowan-events
        "[fn:] invalid\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 14)))))
      (check-org-ast-with parse-org-rowan-events
        "[fn:n] body\n\n* H\n"
        (OrgFile
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 12)))
          (OrgTextLine (TextLine 12 13)))
         (OrgSection
          (OrgHeadline (HeadlineLine 13 14) (HeadlineTrivia 14 15)
                       (HeadlineTitle 15 16) (HeadlineTrivia 16 17))))))
    (test-case "footnote definitions retain one blank and end before two blanks"
      (check-org-ast-with parse-org-rowan-events
        "[fn:a] first\n\ncontinued\n"
        (OrgFile
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 13))
                        (OrgTextLine (TextLine 13 14)))
          (OrgParagraph (OrgTextLine (TextLine 14 24))))))
      (check-org-ast-with parse-org-rowan-events
        "[fn:a] first\n\n\noutside\n"
        (OrgFile
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 13))))
         (OrgTextLine (TextLine 13 14))
         (OrgTextLine (TextLine 14 15))
         (OrgParagraph (OrgTextLine (TextLine 15 23))))))
    (test-case "next definition and EOF flush terminate the preceding footnote"
      (check-org-ast-with parse-org-rowan-events
        "[fn:a] one\n[fn:b] two\n"
        (OrgFile
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 11))))
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 11 15)
          (FootnoteDefinitionLabel 15 16)
          (FootnoteDefinitionDelimiter 16 17)
          (OrgParagraph (OrgTextLine (TextLine 17 22))))))
      (check-org-ast-with parse-org-rowan-events
        "[fn:a] one\n\n"
        (OrgFile
         (OrgFootnoteDefinition
         (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 11)))
          (OrgTextLine (TextLine 11 12)))))
      (check-org-ast-with parse-org-rowan-events
        "[fn:a] one\n\n[fn:b] two\n"
        (OrgFile
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 0 4)
          (FootnoteDefinitionLabel 4 5)
          (FootnoteDefinitionDelimiter 5 6)
          (OrgParagraph (OrgTextLine (TextLine 6 11)))
          (OrgTextLine (TextLine 11 12)))
         (OrgFootnoteDefinition
          (FootnoteDefinitionDelimiter 12 16)
          (FootnoteDefinitionLabel 16 17)
          (FootnoteDefinitionDelimiter 17 18)
          (OrgParagraph (OrgTextLine (TextLine 18 23)))))))
    (test-case "statistics cookies accept Org's percent and fraction shapes"
      (check-org-ast-with parse-org-rowan-events
        "a [50%] [2/3] [%] [/] z\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgStatisticsCookie (StatisticsCookieValue 2 7))
           (TextLine 7 8)
           (OrgStatisticsCookie (StatisticsCookieValue 8 13))
           (TextLine 13 14)
           (OrgStatisticsCookie (StatisticsCookieValue 14 17))
           (TextLine 17 18)
           (OrgStatisticsCookie (StatisticsCookieValue 18 21))
           (TextLine 21 24)))))
      (check-org-ast-with parse-org-rowan-events
        "[5] [5/a] [5%%] x\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 18))))))
    (test-case "line break is only an unescaped pair at the physical line end"
      (check-org-ast-with parse-org-rowan-events
        "a\\\\  \r\nnext\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 1)
                       (OrgLineBreak (LineBreakText 1 7))
                       (TextLine 7 12)))))
      (check-org-ast-with parse-org-rowan-events
        "a\\\\ x\na\\\\\\\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine (TextLine 0 11))))))
    (test-case "inline code and verbatim preserve delimiters and source values"
      (check-org-ast-with parse-org-rowan-events
        "a ~code~ =verb= z\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgCode (InlineMarkupDelimiter 2 3)
                    (InlineMarkupValue 3 7)
                    (InlineMarkupDelimiter 7 8))
           (TextLine 8 9)
           (OrgVerbatim (InlineMarkupDelimiter 9 10)
                        (InlineMarkupValue 10 14)
                        (InlineMarkupDelimiter 14 15))
           (TextLine 15 18)))))
      (check-org-ast-with parse-org-rowan-events
        "x~y~ ~unclosed\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 15)))))
      (check-org-ast-with parse-org-rowan-events
        "~β~\r\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCode (InlineMarkupDelimiter 0 1)
                    (InlineMarkupValue 1 3)
                    (InlineMarkupDelimiter 3 4))
           (TextLine 4 6)))))
      (check-org-ast-with parse-org-rowan-events
        "!~x~ ~a ~ ~b~c\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 15))))))
    (test-case "emphasis Objects use the same Scheme boundary strategy"
      (check-org-ast-with parse-org-rowan-events
        "*bold* /italic/ _under_ +strike+\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgBold (InlineMarkupDelimiter 0 1)
                    (InlineMarkupValue 1 5)
                    (InlineMarkupDelimiter 5 6))
           (TextLine 6 7)
           (OrgItalic (InlineMarkupDelimiter 7 8)
                      (InlineMarkupValue 8 14)
                      (InlineMarkupDelimiter 14 15))
           (TextLine 15 16)
           (OrgUnderline (InlineMarkupDelimiter 16 17)
                         (InlineMarkupValue 17 22)
                         (InlineMarkupDelimiter 22 23))
           (TextLine 23 24)
           (OrgStrikeThrough (InlineMarkupDelimiter 24 25)
                             (InlineMarkupValue 25 31)
                             (InlineMarkupDelimiter 31 32))
           (TextLine 32 33)))))
      (check-org-ast-with parse-org-rowan-events
        "x*y* *open\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 11)))))
      (check-org-ast-with parse-org-rowan-events
        "a *bo\nld* z\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgBold (InlineMarkupDelimiter 2 3)
                    (InlineMarkupValue 3 8)
                    (InlineMarkupDelimiter 8 9))
           (TextLine 9 12))))))
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
                          (SourceHeaderTrivia 25 26)
                          (TextLine 26 34) (BlockEndLine 34 44))
          (OrgSection (OrgHeadline (HeadlineLine 44 46)
                                    (HeadlineTrivia 46 47)
                                    (HeadlineTitle 47 52)
                                    (HeadlineTrivia 52 53)))))))
    (test-case "source block arguments retain Scheme-owned keys and quoted values"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust :results output :var \"hello world\"\nbody\n#+end_src\n"
        (OrgFile
         (OrgSourceBlock
          (BlockBeginLine 0 11) (BlockHeaderTrivia 11 12)
          (SourceLanguage 12 16)
          (SourceHeaderTrivia 16 18) (SourceHeaderKey 18 25)
          (SourceHeaderTrivia 25 26) (SourceHeaderValue 26 32)
          (SourceHeaderTrivia 32 34) (SourceHeaderKey 34 37)
          (SourceHeaderTrivia 37 38) (SourceHeaderValue 38 51)
          (SourceHeaderTrivia 51 52)
          (TextLine 52 57) (BlockEndLine 57 67)))))
    (test-case "header-like text without a leading separator remains trivia"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust x:bad :var ok\nbody\n#+end_src\n"
        (OrgFile
         (OrgSourceBlock
          (BlockBeginLine 0 11) (BlockHeaderTrivia 11 12)
          (SourceLanguage 12 16)
          (SourceHeaderTrivia 16 24) (SourceHeaderKey 24 27)
          (SourceHeaderTrivia 27 28) (SourceHeaderValue 28 30)
          (SourceHeaderTrivia 30 31)
          (TextLine 31 36) (BlockEndLine 36 46)))))
    (test-case "unterminated blocks recover as text before the next heading"
      (check-org-ast-with parse-org-rowan-events
        "* Parent\n#+BEGIN_SRC\n** body\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 8) (HeadlineTrivia 8 9))
          (OrgParagraph (OrgTextLine (TextLine 9 21)))
          (OrgSection
           (OrgHeadline (HeadlineLine 21 23) (HeadlineTrivia 23 24)
                        (HeadlineTitle 24 28) (HeadlineTrivia 28 29)))))))
    (test-case "unclosed recursive blocks preserve later headline structure"
      (check-org-ast-with parse-org-rowan-events
        "* First\n#+begin_quote\nunclosed\n** Next\nvisible\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 7) (HeadlineTrivia 7 8))
          (OrgParagraph (OrgTextLine (TextLine 8 31)))
          (OrgSection
           (OrgHeadline (HeadlineLine 31 33) (HeadlineTrivia 33 34)
                        (HeadlineTitle 34 38) (HeadlineTrivia 38 39))
           (OrgParagraph (OrgTextLine (TextLine 39 47))))))))
    (test-case "malformed property body recovers as text, not a named drawer"
      (check-org-ast-with parse-org-rowan-events
        "* Parent\n:PROPERTIES:\n:ID: one\nmalformed\n:END:\n** Next\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 8) (HeadlineTrivia 8 9))
          (OrgParagraph (OrgTextLine (TextLine 9 47)))
          (OrgSection
           (OrgHeadline (HeadlineLine 47 49) (HeadlineTrivia 49 50)
                        (HeadlineTitle 50 54) (HeadlineTrivia 54 55)))))))
    (test-case "adjacent Org comments form one typed element"
      (check-org-ast-with parse-org-rowan-events
        "# one\n# two\ntext\n#\n"
        (OrgFile
         (OrgComment (CommentLine 0 6) (CommentLine 6 12))
         (OrgParagraph (OrgTextLine (TextLine 12 17)))
         (OrgComment (CommentLine 17 19)))))
    (test-case "indented comments remain inside the owning list item"
      (check-org-ast-with parse-org-rowan-events
        "- item\n  # child\n  # next\n- peer\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (OrgParagraph (OrgTextLine (TextLine 2 7)))
           (OrgComment (CommentLine 7 17) (CommentLine 17 26)))
          (OrgListItem
           (ListBullet 26 27) (ListTrivia 27 28)
           (OrgParagraph (OrgTextLine (TextLine 28 33))))))))
    (test-case "hash-prefixed text and keywords are not comments"
      (check-org-ast-with parse-org-rowan-events
        "#not-comment\n#+TITLE: Yes\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 13)))
         (OrgKeyword
          (KeywordTrivia 13 15) (KeywordKey 15 20)
          (KeywordTrivia 20 22) (KeywordValue 22 25)
          (KeywordTrivia 25 26)))))
    (test-case "diary S-expressions are standalone source-backed Elements"
      (check-org-ast-with parse-org-rowan-events
        "%%(diary-anniversary 1 1 2000)\ntext\n%%not-diary\n"
        (OrgFile
         (OrgDiarySexp (DiarySexpValue 0 30) (DiarySexpTrivia 30 31))
         (OrgParagraph (OrgTextLine (TextLine 31 48)))))
      (check-org-ast-with parse-org-rowan-events
        "%%(x) \r\n"
        (OrgFile
         (OrgDiarySexp (DiarySexpValue 0 6) (DiarySexpTrivia 6 8)))))
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
    (test-case "indented property drawers preserve keys and trivia"
      (check-org-ast-with parse-org-rowan-events
        "* H\n  :PROPERTIES:\n  :ID: x\n  :END:\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPropertyDrawer
           (DrawerBeginLine 4 19)
           (OrgNodeProperty
            (PropertyTrivia 19 22) (PropertyKey 22 24)
            (PropertyTrivia 24 26) (PropertyValue 26 27)
            (PropertyTrivia 27 28))
           (DrawerEndLine 28 36))))))
    (test-case "property keys scan source bytes until the declared colon"
      (check-org-ast-with parse-org-rowan-events
        "* H\n:PROPERTIES:\n:A+B: yes\n:END:\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPropertyDrawer
           (DrawerBeginLine 4 17)
           (OrgNodeProperty
            (PropertyTrivia 17 18) (PropertyKey 18 21)
            (PropertyTrivia 21 23) (PropertyValue 23 26)
            (PropertyTrivia 26 27))
           (DrawerEndLine 27 33))))))
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
    (test-case "one Planning Element keeps every declared key on its line"
      (check-org-ast-with parse-org-rowan-events
        "* H\nSCHEDULED: <a> DEADLINE: <b>\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPlanning
           (PlanningKey 4 13) (PlanningTrivia 13 15)
           (PlanningValue 15 18) (PlanningTrivia 18 19)
           (PlanningKey 19 27) (PlanningTrivia 27 29)
           (PlanningValue 29 32) (PlanningTrivia 32 33))))))
    (test-case "planning is not promoted after ordinary paragraph content"
      (check-org-ast-with parse-org-rowan-events
        "* H\nbody\nSCHEDULED: later\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgParagraph (OrgTextLine (TextLine 4 26)))))))
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
    (test-case "POO list markers retain nested and sibling item scopes"
      (check-org-ast-with parse-org-rowan-events "- a\n  - b\n- c\n"
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
      (check-org-ast-with parse-org-rowan-events "1. a\n2) b\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 2) (ListTrivia 2 3)
           (OrgParagraph (OrgTextLine (TextLine 3 5))))
          (OrgListItem
           (ListBullet 5 7) (ListTrivia 7 8)
           (OrgParagraph (OrgTextLine (TextLine 8 10))))))))
    (test-case "list continuation and blank line remain within item"
      (check-org-ast-with parse-org-rowan-events "- alpha\n  more\n- beta\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (OrgParagraph
            (OrgTextLine (TextLine 2 15))))
          (OrgListItem
           (ListBullet 15 16) (ListTrivia 16 17)
           (OrgParagraph (OrgTextLine (TextLine 17 22)))))))
      (check-org-ast-with parse-org-rowan-events "- a\n\n- b\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (OrgParagraph (OrgTextLine (TextLine 2 4)))
           (ListTrivia 4 5))
          (OrgListItem
           (ListBullet 5 6) (ListTrivia 6 7)
           (OrgParagraph (OrgTextLine (TextLine 7 9))))))))
    (test-case "Scheme list algorithm emits typed counter checkbox and tag spans"
      (check-org-ast-with parse-org-rowan-events "- [@2] [X] done\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
           (ListTrivia 2 4) (ListCounterValue 4 5) (ListTrivia 5 7)
           (ListTrivia 7 8) (ListCheckboxValue 8 9) (ListTrivia 9 11)
           (OrgParagraph (OrgTextLine (TextLine 11 16)))))))
      (check-org-ast-with parse-org-rowan-events "- term :: body\n"
        (OrgFile
         (OrgPlainList
          (OrgListItem
           (ListBullet 0 1) (ListTrivia 1 2)
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
    (test-case "recursive containers keep nested Scheme-owned Elements"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_quote\ntext\n- item\n#+end_quote\n"
        (OrgFile
         (OrgQuoteBlock
          (BlockBeginLine 0 14)
          (OrgParagraph (OrgTextLine (TextLine 14 19)))
          (OrgPlainList
           (OrgListItem
            (ListBullet 19 20) (ListTrivia 20 21)
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
        (check (vector-length (hash-ref ir "line")) => 1)))))
