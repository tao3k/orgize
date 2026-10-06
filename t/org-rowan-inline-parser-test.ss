;;; -*- Gerbil -*-
;;; The Org-owned algorithm executes as Scheme before AOT lowering.

(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/v1/modules/org-parser/macro-funs.ss"
                 expand-org-macro-template expand-org-property-macros)
        (only-in :clan/poo/object .o)
        (only-in :gerbil-parser/src/modules/parser/line-structure-objects
                 line-structure-blocks)
        (only-in "../languages/org/v1/parser.ss" org-v1-line-structure)
        (only-in "../languages/org/v1/modules/org-parser/types.ss"
                 org-event-block? org-named-block?
                 org-inline-markup? org-inline-script?
                 org-event-helper?
                 org-event-strategy?)
        (only-in "../languages/org/v1/modules/org-parser/objects.ss"
                 make-org-event-block org-event-block-id
                 make-org-named-block
                 make-org-inline-markup org-inline-markup-node
                 make-org-inline-script org-inline-script-node
                 make-org-event-helper org-event-helper-descriptor
                 make-org-event-strategy org-event-strategy-root)
        (only-in "org-parser-test-support.ss" check-org-ast-with org-events-cover-source?)
        (only-in "../languages/org/v1/rowan-event-runtime.ss"
                 parse-org-rowan-events
                 parse-org-rowan-events-with-inlinetask-level
                 parse-org-rowan-events-with-inline-script-policy))
(export org-v1-rowan-inline-parser-test)

(def org-v1-rowan-inline-parser-test
  (test-suite "Org inline source-backed Objects"
    (test-case "native macro templates preserve placeholder and Unicode semantics"
      (for-each
       (lambda (row)
         (check (expand-org-macro-template (car row) '("λ,a" "β")) => (cadr row)))
       '(("" "") ("literal λ" "literal λ") ("$1/$2/$9" "λ,a/β/")
         ("$0::$1::$0" "λ,a, β::λ,a::λ,a, β") ("$$/$x/$" "$/$x/$")
         ("$10 $01 $$$1" "λ,a0 λ,a, β1 $λ,a")))
      (check (expand-org-macro-template "$0/$1" '()) => "/")
      ;; Argument indexing is built once, not traversed for every placeholder.
      (for-each
       (lambda (count)
         (check (expand-org-macro-template
                 (string-join (make-list count "$1") "")
                 (make-list count "x")) => (make-string count #\x)))
       '(1000 10000)))
    (test-case "secondary property macros share native escaped argument events"
      (let (definitions '(("x" . "old") ("x" . "$1/$2/$0")))
        (check (expand-org-property-macros "λ/{{{x(a\\,b,c)}}}/β" definitions)
          => "λ/a,b/c/a,b, c/β")
        (check (expand-org-property-macros "* {{{x( a , b )}}}\r\nβ" definitions)
          => "* a/b/a, b\r\nβ")
        (check (expand-org-property-macros "{{{missing(x)}}} {{{x(abc)" definitions)
          => "{{{missing(x)}}} {{{x(abc)")
        (check (expand-org-property-macros "={{{x(a,b)}}}=" definitions)
          => "={{{x(a,b)}}}=")))
    (test-case "macro escaped commas are native argument content"
      (check-org-ast-with parse-org-rowan-events "{{{x(a\\,b,c)}}}\n"
        (OrgFile (OrgParagraph (OrgTextLine
          (OrgMacro (MacroDelimiter 0 3) (MacroName 3 4) (MacroDelimiter 4 5)
            (MacroArguments
              (MacroArgumentContent (OrgMacroArgument
                (MacroArgumentText 5 6) (MacroTrivia 6 7) (MacroArgumentText 7 9)))
              (MacroTrivia 9 10)
              (MacroArgumentContent (OrgMacroArgument (MacroArgumentText 10 11))))
            (MacroDelimiter 11 15)) (TextLine 15 16))))))
    (test-case "POO strategy declarations reject untyped rule tuples"
      (let* ((block (make-org-event-block
                    1 (car (line-structure-blocks org-v1-line-structure))))
            (markup (make-org-inline-markup 42 3 'OrgBold))
            (script (make-org-inline-script 94 2 'OrgSuperscript))
            (helper (make-org-event-helper
                     'source-fragment '((seen #f)) '((finish-node))))
            (strategy (make-org-event-strategy
                       'OrgFile '() '((finish-node)) '() (list helper))))
        (check (org-event-block? block) => #t)
        (check (org-named-block?
                (make-org-named-block "#+BEGIN_" "#+END_"
                                      'OrgSpecialBlock 'SpecialBlockName)) => #t)
        (check (org-event-block-id block) => 1)
        (check (org-inline-markup? markup) => #t)
        (check (org-inline-markup-node markup) => 'OrgBold)
        (check (org-inline-script? script) => #t)
        (check (org-inline-script-node script) => 'OrgSuperscript)
        (check (org-inline-script? '(94 2 OrgSuperscript)) => #f)
        (check (org-event-helper? helper) => #t)
        (check (org-event-helper-descriptor helper)
               => '(source-fragment ((seen #f)) ((finish-node))))
        (check (org-event-helper? '(source-fragment () ())) => #f)
        (check (org-event-strategy? strategy) => #t)
        (check (org-event-strategy-root strategy) => 'OrgFile)
        (check (org-event-block? '(1 . block)) => #f)
        (check (org-named-block? '("#+BEGIN_" . "#+END_")) => #f)
        (check (org-inline-markup? '(42 3 OrgBold)) => #f)
        (check (org-inline-markup?
                (.o kind: 'org-inline-markup byte: 256 id: 3
                    node: 'OrgBold))
               => #f)))
    (test-case "Scheme parses source-backed LaTeX math fragments"
      (check-org-ast-with parse-org-rowan-events
        "a \\(x\\) b\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 2)
           (OrgLaTeXFragment (LatexFragmentValue 2 7))
           (TextLine 7 10)))))
      (check-org-ast-with parse-org-rowan-events
        "\\[x\\] $$y$$ $z$\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgLaTeXFragment (LatexFragmentValue 0 5))
           (TextLine 5 6)
           (OrgLaTeXFragment (LatexFragmentValue 6 11))
           (TextLine 11 12)
           (OrgLaTeXFragment (LatexFragmentValue 12 15))
           (TextLine 15 16)))))
      (check-org-ast-with parse-org-rowan-events
        "$ x$ $x $\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 10)))))
      (check-org-ast-with parse-org-rowan-events
        "$unfinished [[id:x]]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 12)
           (OrgLink (LinkTrivia 12 14) (LinkTarget 14 18)
                    (LinkTrivia 18 20))
           (TextLine 20 21))))))
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
    (test-case "inlinetask Element keeps its END and leaves following outline intact"
      (check-org-ast-with parse-org-rowan-events
        "*************** TODO Inline\nBody.\n*************** END\n* Next\n"
        (OrgFile
         (OrgInlinetask
          (OrgHeadline
           (HeadlineLine 0 15) (HeadlineTrivia 15 16)
           (HeadlineTitle 16 27) (HeadlineTrivia 27 28))
          (OrgParagraph (OrgTextLine (TextLine 28 34)))
          (OrgInlinetaskEnd (HeadlineLine 34 49)
                            (InlinetaskEndLine 49 54)))
         (OrgSection
          (OrgHeadline
           (HeadlineLine 54 55) (HeadlineTrivia 55 56)
           (HeadlineTitle 56 60) (HeadlineTrivia 60 61)))))
      (check-org-ast-with parse-org-rowan-events
        "*************** Note\nAfter text.\n"
        (OrgFile
         (OrgInlinetask
          (OrgHeadline
           (HeadlineLine 0 15) (HeadlineTrivia 15 16)
           (HeadlineTitle 16 20) (HeadlineTrivia 20 21)))
         (OrgParagraph (OrgTextLine (TextLine 21 33))))))
    (test-case "configured inlinetask threshold is Scheme algorithm state"
      (check-org-ast-with
       (lambda (source)
         (parse-org-rowan-events-with-inlinetask-level source 4))
       "**** Inline\nBody.\n**** END\n"
       (OrgFile
        (OrgInlinetask
         (OrgHeadline
          (HeadlineLine 0 4) (HeadlineTrivia 4 5)
          (HeadlineTitle 5 11) (HeadlineTrivia 11 12))
         (OrgParagraph (OrgTextLine (TextLine 12 18)))
         (OrgInlinetaskEnd (HeadlineLine 18 22)
                           (InlinetaskEndLine 22 27)))))
      (check-org-ast-with parse-org-rowan-events
        "**** Inline\n"
        (OrgFile
         (OrgSection
          (OrgHeadline
           (HeadlineLine 0 4) (HeadlineTrivia 4 5)
           (HeadlineTitle 5 11) (HeadlineTrivia 11 12))))))
    (test-case "Scheme cloze Objects retain text, hint, and identifier spans"
      (check-org-ast-with parse-org-rowan-events
        "{{text}}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCloze (ClozeDelimiter 0 2)
                     (OrgClozeText (OrgTextLine (TextLine 2 6)))
                     (ClozeDelimiter 6 7) (ClozeDelimiter 7 8))
           (TextLine 8 9)))))
      (check-org-ast-with parse-org-rowan-events
        "{{text}@id}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCloze (ClozeDelimiter 0 2)
                     (OrgClozeText (OrgTextLine (TextLine 2 6)))
                     (ClozeDelimiter 6 7) (ClozeDelimiter 7 8)
                     (ClozeId 8 10) (ClozeDelimiter 10 11))
           (TextLine 11 12)))))
      (check-org-ast-with parse-org-rowan-events
        "{{*text*}{hint}@card-id}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCloze
            (ClozeDelimiter 0 2)
            (OrgClozeText
             (OrgTextLine
              (OrgBold (InlineMarkupDelimiter 2 3)
                       (InlineMarkupValue 3 7)
                       (InlineMarkupDelimiter 7 8))))
            (ClozeDelimiter 8 9) (ClozeDelimiter 9 10)
            (ClozeHint 10 14) (ClozeDelimiter 14 15)
            (ClozeDelimiter 15 16) (ClozeId 16 23)
            (ClozeDelimiter 23 24))
           (TextLine 24 25)))))
      (check-org-ast-with parse-org-rowan-events
        "{{a[b]}}\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCloze (ClozeDelimiter 0 2)
                     (ClozeText 2 6)
                     (ClozeDelimiter 6 7) (ClozeDelimiter 7 8))
           (TextLine 8 9))))))
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
                     (MacroDelimiter 8 9)
                     (MacroArguments (MacroArgumentContent (OrgMacroArgument (MacroArgumentText 9 11))))
                     (MacroDelimiter 11 15))
           (TextLine 15 16)))))
      (check-org-ast-with parse-org-rowan-events
        "{{{9bad}}} {{{broken\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 21))))))
    (test-case "Scheme citation-reference Objects retain source-backed fields"
      (check-org-ast-with parse-org-rowan-events
        "[cite:@ok; @].\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation
            (OrgCitationHead (CitationDelimiter 0 6))
            (OrgCitationReference
             (CitationReferenceMarker 6 7)
             (CitationReferenceKey 7 9))
            (CitationSeparator 9 10)
            (OrgCitationMalformedReference
             (CitationMalformedSegment 10 12))
            (CitationDelimiter 12 13))
           (TextLine 13 15)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite:@doe2020]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation (OrgCitationHead (CitationDelimiter 0 6))
                        (OrgCitationReference
                         (CitationReferenceMarker 6 7)
                         (CitationReferenceKey 7 14))
                        (CitationDelimiter 14 15))
           (TextLine 15 16)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite/:@key]\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 13)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite:\\@key] [cite:@ ]\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 23)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite:@key] [cite:no key]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation (OrgCitationHead (CitationDelimiter 0 6))
                        (OrgCitationReference
                         (CitationReferenceMarker 6 7)
                         (CitationReferenceKey 7 10))
                        (CitationDelimiter 10 11))
           (TextLine 11 26)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite:see [p. 12] @key]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation (OrgCitationHead (CitationDelimiter 0 6))
                        (OrgCitationReference
                         (OrgCitationReferencePrefix
                          (CitationReferencePrefixContent
                           (OrgTextLine (TextLine 6 18))))
                         (CitationReferenceMarker 18 19)
                         (CitationReferenceKey 19 22))
                        (CitationDelimiter 22 23))
           (TextLine 23 24)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite/text:see @doe2020 p. 42; cf. @roe2021]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation
            (OrgCitationHead (CitationDelimiter 0 6)
                             (CitationStyle 6 10)
                             (CitationDelimiter 10 11))
            (OrgCitationReference
             (OrgCitationReferencePrefix
              (CitationReferencePrefixContent
               (OrgTextLine (TextLine 11 15))))
             (CitationReferenceMarker 15 16)
             (CitationReferenceKey 16 23)
             (OrgCitationReferenceSuffix
              (TextLine 23 24)
              (CitationReferenceSuffixContent
               (OrgTextLine (TextLine 24 29)))))
            (CitationSeparator 29 30)
            (OrgCitationReference
             (OrgCitationReferencePrefix
              (TextLine 30 31)
              (CitationReferencePrefixContent
               (OrgTextLine (TextLine 31 35))))
             (CitationReferenceMarker 35 36)
             (CitationReferenceKey 36 43))
            (CitationDelimiter 43 44))
           (TextLine 44 45)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite:see;@key;and]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation
            (OrgCitationHead (CitationDelimiter 0 6))
            (OrgCitationGlobalPrefix
             (CitationGlobalPrefixContent
              (OrgTextLine (TextLine 6 9))))
            (CitationSeparator 9 10)
            (OrgCitationReference
             (CitationReferenceMarker 10 11)
             (CitationReferenceKey 11 14))
            (CitationSeparator 14 15)
            (OrgCitationGlobalSuffix
             (CitationGlobalSuffixContent
              (OrgTextLine (TextLine 15 18))))
            (CitationDelimiter 18 19))
           (TextLine 19 20)))))
      (check-org-ast-with parse-org-rowan-events
        "[cite:@key\n[cite:@next]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 11)
           (OrgCitation (OrgCitationHead (CitationDelimiter 11 17))
                        (OrgCitationReference
                         (CitationReferenceMarker 17 18)
                         (CitationReferenceKey 18 22))
                        (CitationDelimiter 22 23))
           (TextLine 23 24))))))
    (test-case "citation header components are native Scheme source spans"
      (check-org-ast-with parse-org-rowan-events
        "[cite/noauthor/bare:@key]\r\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation
            (OrgCitationHead (CitationDelimiter 0 6)
                             (CitationStyle 6 14)
                             (CitationDelimiter 14 15)
                             (CitationVariant 15 19)
                             (CitationDelimiter 19 20))
            (OrgCitationReference
             (CitationReferenceMarker 20 21)
             (CitationReferenceKey 21 24))
            (CitationDelimiter 24 25))
           (TextLine 25 27)))))
      (for-each
       (lambda (source)
         (let (events (parse-org-rowan-events source))
           (check (org-events-cover-source? source events) => #t)
           (check (filter (lambda (event)
                            (and (eq? (car event) 'start)
                                 (eq? (cadr event) 'OrgCitation)))
                          events) => '())))
       '("[cite/:@key]\r\n" "[cite/text/:@key]\n"
         "[cite/text//bare:@key]\r\n" "[cite/text bare:@key]\n"
         "[cite/text:no key]\n")))
    (test-case "citation affix content ranges are native Scheme decisions"
      (check-org-ast-with parse-org-rowan-events
        "[cite: \t; \t@key \t; \t]\r\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgCitation
            (OrgCitationHead (CitationDelimiter 0 6))
            (OrgCitationGlobalPrefix (TextLine 6 8))
            (CitationSeparator 8 9)
            (OrgCitationReference
             (OrgCitationReferencePrefix (TextLine 9 11))
             (CitationReferenceMarker 11 12)
             (CitationReferenceKey 12 15)
             (OrgCitationReferenceSuffix (TextLine 15 17)))
            (CitationSeparator 17 18)
            (OrgCitationGlobalSuffix (TextLine 18 20))
            (CitationDelimiter 20 21))
           (TextLine 21 23))))))
    (test-case "timestamp dates are Scheme-owned source-backed Objects"
      (check-org-ast-with parse-org-rowan-events
        "<2026-09-23 Wed>\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTimestampActive
            (OrgTimestampPoint
             (TimestampDelimiter 0 1)
             (TimestampDate (TimestampYear 1 5)
                            (TimestampDateSeparator 5 6)
                            (TimestampMonth 6 8)
                            (TimestampDateSeparator 8 9)
                            (TimestampDay 9 11))
             (TimestampTrivia 11 12)
             (TimestampDayName 12 15)
             (TimestampDelimiter 15 16)))
           (TextLine 16 17)))))
      (check-org-ast-with parse-org-rowan-events
        "[2026-09-23]--[2026-09-24]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTimestampInactive
            (OrgTimestampPoint
             (TimestampDelimiter 0 1)
             (TimestampDate (TimestampYear 1 5)
                            (TimestampDateSeparator 5 6)
                            (TimestampMonth 6 8)
                            (TimestampDateSeparator 8 9)
                            (TimestampDay 9 11))
             (TimestampDelimiter 11 12))
            (TimestampRangeSeparator 12 14)
            (OrgTimestampPoint
             (TimestampDelimiter 14 15)
             (TimestampDate (TimestampSecondYear 15 19)
                            (TimestampDateSeparator 19 20)
                            (TimestampSecondMonth 20 22)
                            (TimestampDateSeparator 22 23)
                            (TimestampSecondDay 23 25))
             (TimestampDelimiter 25 26)))
           (TextLine 26 27)))))
      (check-org-ast-with parse-org-rowan-events
        "[2026-09-23]-[2026-09-24]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTimestampInactive
            (OrgTimestampPoint
             (TimestampDelimiter 0 1)
             (TimestampDate (TimestampYear 1 5)
                            (TimestampDateSeparator 5 6)
                            (TimestampMonth 6 8)
                            (TimestampDateSeparator 8 9)
                            (TimestampDay 9 11))
             (TimestampDelimiter 11 12))
            (TimestampRangeSeparator 12 13)
            (OrgTimestampPoint
             (TimestampDelimiter 13 14)
             (TimestampDate (TimestampSecondYear 14 18)
                            (TimestampDateSeparator 18 19)
                            (TimestampSecondMonth 19 21)
                            (TimestampDateSeparator 21 22)
                            (TimestampSecondDay 22 24))
             (TimestampDelimiter 24 25)))
           (TextLine 25 26)))))
      (check-org-ast-with parse-org-rowan-events
        "<2026-09-23 Wed 10:00-11:00 ++1w -2d>\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTimestampActive
            (OrgTimestampPoint
             (TimestampDelimiter 0 1)
             (TimestampDate (TimestampYear 1 5)
                            (TimestampDateSeparator 5 6)
                            (TimestampMonth 6 8)
                            (TimestampDateSeparator 8 9)
                            (TimestampDay 9 11))
             (TimestampTrivia 11 12)
             (TimestampDayName 12 15)
             (TimestampTrivia 15 16)
             (TimestampTime
              (TimestampHour 16 18) (TimestampTimeSeparator 18 19)
              (TimestampMinute 19 21) (TimestampTimeRangeSeparator 21 22)
              (TimestampTimeEnd
               (TimestampEndHour 22 24) (TimestampTimeSeparator 24 25)
               (TimestampEndMinute 25 27)))
             (TimestampTrivia 27 28)
             (TimestampRepeater (TimestampRepeaterMark 28 30)
                                (TimestampRepeaterValue 30 31)
                                (TimestampRepeaterUnit 31 32))
             (TimestampTrivia 32 33)
             (TimestampDelay (TimestampDelayMark 33 34)
                             (TimestampDelayValue 34 35)
                             (TimestampDelayUnit 35 36))
             (TimestampDelimiter 36 37)))
           (TextLine 37 38)))))
      (check-org-ast-with parse-org-rowan-events
        "<%%(diary-float t 1 2)>\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTimestampDiary
            (TimestampDelimiter 0 1)
            (TimestampDiaryExpression 1 22)
            (TimestampDelimiter 22 23))
           (TextLine 23 24)))))
      (check-org-ast-with parse-org-rowan-events
        "<%%(diary-float t 4 2) 12:00-14:00>\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgTimestampDiary
            (TimestampDelimiter 0 1)
            (TimestampDiaryExpression 1 22)
            (TimestampTrivia 22 23)
            (TimestampTime
             (TimestampHour 23 25) (TimestampTimeSeparator 25 26)
             (TimestampMinute 26 28) (TimestampTimeRangeSeparator 28 29)
             (TimestampTimeEnd
              (TimestampEndHour 29 31) (TimestampTimeSeparator 31 32)
              (TimestampEndMinute 32 34)))
            (TimestampDelimiter 34 35))
           (TextLine 35 36))))))
    (test-case "timestamp clocks and range endpoints are classified by Scheme"
      (let* ((source "<2026-09-23 9:05-10:06>\r\n")
             (events (parse-org-rowan-events source))
             (components
              (filter (lambda (event)
                        (and (eq? (car event) 'token)
                             (memq (cadr event)
                                   '(TimestampHour TimestampMinute
                                     TimestampEndHour TimestampEndMinute
                                     TimestampTimeRangeSeparator)))) events)))
        (check (org-events-cover-source? source events) => #t)
        (check (map (lambda (event)
                      (list (cadr event) (caddr event) (cadddr event))) components)
               => '((TimestampHour 12 13) (TimestampMinute 14 16)
                    (TimestampTimeRangeSeparator 16 17)
                    (TimestampEndHour 17 19) (TimestampEndMinute 20 22))))
      (for-each
       (lambda (clock)
         (let* ((source (string-append "<2026-09-23 " clock ">\n"))
                (events (parse-org-rowan-events source)))
           (check (org-events-cover-source? source events) => #t)
           (check (filter (lambda (event)
                            (and (eq? (car event) 'token)
                                 (memq (cadr event)
                                       '(TimestampHour TimestampMinute)))) events)
                  => '())))
       '("10:" ":05" "10::05")))
    (test-case "timestamp cookie admission is native Scheme, including recovery"
      (for-each
       (lambda (cookie)
         (let* ((source (string-append "<2026-09-23 " cookie ">\r\n"))
                (events (parse-org-rowan-events source))
                (cookies (filter (lambda (event)
                                   (and (eq? (car event) 'start)
                                        (memq (cadr event)
                                              '(TimestampRepeater TimestampDelay))))
                                 events)))
           (check (org-events-cover-source? source events) => #t)
           (check (map cadr cookies)
                  => (list (if (char=? (string-ref cookie 0) #\-)
                             'TimestampDelay 'TimestampRepeater)))
           (check (map caddr (filter (lambda (event)
                                     (and (eq? (car event) 'token)
                                          (memq (cadr event)
                                                '(TimestampRepeaterMark TimestampDelayMark))))
                                   events)) => '(12))))
       '("+1h" "++12d" ".+3w" "+4m" "+5y" "-2d" "--4w"))
      (for-each
       (lambda (cookie)
         (let* ((source (string-append "<2026-09-23 " cookie ">\n"))
                (events (parse-org-rowan-events source)))
           (check (org-events-cover-source? source events) => #t)
           (check (filter (lambda (event)
                            (and (eq? (car event) 'start)
                                 (memq (cadr event)
                                       '(TimestampRepeater TimestampDelay))))
                          events) => '())))
       '("+" "++w" ".1w" "+-1w" "+1ww" "+1.2w" "---2d" "-d" "-2" "+1q")))
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
         (OrgFixedWidth (FixedWidthPrefix 6 8) (FixedWidthValue 8 10)
                        (FixedWidthPrefix 10 11) (FixedWidthValue 11 12)
                        (FixedWidthPrefix 12 14) (FixedWidthValue 14 16))
         (OrgParagraph (OrgTextLine (TextLine 16 21))))))
    (test-case "POO-declared inline links preserve descriptions and malformed text"
      (check-org-ast-with parse-org-rowan-events
        "go [[https://a][α]] and [[id:b]]\n[[broken\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 3)
           (OrgLink (LinkTrivia 3 5) (LinkTarget 5 14)
                    (LinkTrivia 14 16)
                    (OrgLinkDescription (OrgTextLine (TextLine 16 18)))
                    (LinkTrivia 18 20))
           (TextLine 20 25)
           (OrgLink (LinkTrivia 25 27) (LinkTarget 27 31)
                    (LinkTrivia 31 33))
           (TextLine 33 43))))))
    (test-case "angle and plain URLs are Scheme-owned link Objects"
      (check-org-ast-with parse-org-rowan-events
        "Visit <https://example.com/path>.\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 6)
           (OrgLink (LinkTrivia 6 7) (LinkTarget 7 31)
                    (LinkTrivia 31 32))
           (TextLine 32 34)))))
      (check-org-ast-with parse-org-rowan-events
        "Visit https://example.com/path.\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 6) (OrgLink (LinkTarget 6 30))
           (TextLine 30 32))))))
    (test-case "nested link descriptions keep URL text without recursive links"
      (check-org-ast-with parse-org-rowan-events
        "go [[id:a][https://example.org]]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 3)
           (OrgLink (LinkTrivia 3 5) (LinkTarget 5 9)
                    (LinkTrivia 9 11)
                    (OrgLinkDescription (OrgTextLine (TextLine 11 30)))
                    (LinkTrivia 30 32))
           (TextLine 32 33))))))
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
        "[fn:n:See *bold* text]\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (OrgFootnoteReference
            (FootnoteReferenceDelimiter 0 4)
            (FootnoteReferenceLabel 4 5)
            (FootnoteReferenceDelimiter 5 6)
            (OrgFootnoteInlineDefinition
             (OrgTextLine
              (TextLine 6 10)
              (OrgBold (InlineMarkupDelimiter 10 11)
                       (InlineMarkupValue 11 15)
                       (InlineMarkupDelimiter 15 16))
              (TextLine 16 21)))
            (FootnoteReferenceDelimiter 21 22))
           (TextLine 22 23)))))
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
      ;; Org Mode's object dispatcher distinguishes the lowercase `f` here.
      (check-org-ast-with parse-org-rowan-events
        "x [Fn:n] y\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 11)))))
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
          (OrgTextLine
           (TextLine 0 9)
           (OrgSubscript (InlineScriptDelimiter 9 10)
                         (InlineScriptValue 10 14))
           (TextLine 14 18))))))
    (test-case "Scheme script Objects preserve delimiters and nested braces"
      (check-org-ast-with parse-org-rowan-events
        "x_abc y^2\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 1)
           (OrgSubscript (InlineScriptDelimiter 1 2)
                         (InlineScriptValue 2 5))
           (TextLine 5 7)
           (OrgSuperscript (InlineScriptDelimiter 7 8)
                           (InlineScriptValue 8 9))
           (TextLine 9 10)))))
      (check-org-ast-with parse-org-rowan-events
        "x_{a{b}c} y^*\n"
        (OrgFile
         (OrgParagraph
          (OrgTextLine
           (TextLine 0 1)
           (OrgSubscript (InlineScriptDelimiter 1 3)
                         (InlineScriptValue 3 8)
                         (InlineScriptDelimiter 8 9))
           (TextLine 9 11)
           (OrgSuperscript (InlineScriptDelimiter 11 12)
                           (InlineScriptValue 12 13))
           (TextLine 13 14)))))
      (check-org-ast-with parse-org-rowan-events
        "AB_2O x^a,\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 11)))))
      (check-org-ast-with parse-org-rowan-events
        "_abc\n"
        (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 5))))))
    (test-case "Scheme inline helper inherits configured script policy"
      (check-org-ast-with
       (lambda (source)
         (parse-org-rowan-events-with-inline-script-policy source 0))
       "x_abc y_{z}\n"
       (OrgFile (OrgParagraph (OrgTextLine (TextLine 0 12)))))
      (check-org-ast-with
       (lambda (source)
         (parse-org-rowan-events-with-inline-script-policy source 1))
       "x_abc y_{z}\n"
       (OrgFile
        (OrgParagraph
         (OrgTextLine
          (TextLine 0 7)
          (OrgSubscript (InlineScriptDelimiter 7 9)
                        (InlineScriptValue 9 10)
                        (InlineScriptDelimiter 10 11))
          (TextLine 11 12))))))
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
        "[FN:n] body\n* H\n"
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
  ))
