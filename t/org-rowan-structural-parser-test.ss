;;; -*- Gerbil -*-
;;; The Org-owned algorithm executes as Scheme before AOT lowering.

(import (only-in :std/test check test-case test-suite)
        (only-in :clan/poo/object .o)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in :std/misc/ports read-all-as-string)
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
        (only-in "org-parser-test-support.ss"
                 check-org-ast-with org-events-cover-source?)
        (only-in "../languages/org/v1/rowan-event-fixture.ss" rowan-event-fixture-json)
        (only-in "../languages/org/v1/rowan-event-parser.ss"
                 parse-org-rowan-events
                 parse-org-rowan-events-with-inlinetask-level
                 parse-org-rowan-events-with-inline-script-policy
                 parse_org_rowan_events))
(export org-v1-rowan-structural-parser-test)

(def org-v1-rowan-structural-parser-test
  (test-suite "Org structural Elements and metadata"
    (test-case "TAGS vocabulary is Scheme-owned source-backed structure"
      (check-org-ast-with parse-org-rowan-events
        "#+TAGS: { @work(w) @home(h) }\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 6) (KeywordTrivia 6 7)
          (OrgKeywordRawValue
           (KeywordTrivia 7 8)
           (OrgTagVocabulary
            (OrgTagExclusiveGroup
             (TagGroupOpen 8 9) (TagTrivia 9 10)
             (TagName 10 15) (TagShortcutOpen 15 16)
             (TagShortcut 16 17) (TagShortcutClose 17 18)
             (TagTrivia 18 19) (TagName 19 24)
             (TagShortcutOpen 24 25) (TagShortcut 25 26)
             (TagShortcutClose 26 27) (TagTrivia 27 28)
             (TagGroupClose 28 29))))
          (KeywordTrivia 29 30)))))
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
                          (OrgBlockBodyLine (TextLine 26 34))
                          (BlockEndLine 34 44))
          (OrgSection (OrgHeadline (HeadlineLine 44 46)
                                    (HeadlineTrivia 46 47)
                                    (OrgHeadlineTitle
                                     (OrgTextLine (TextLine 47 52)))
                                    (HeadlineTrivia 52 53)))))))
    (test-case "source block escape is Scheme-owned and retains raw lines"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src\n,* hi\n,#+foo\n#+end_src\n"
        (OrgFile
         (OrgSourceBlock
          (BlockBeginLine 0 11) (BlockHeaderTrivia 11 12)
          (OrgBlockBodyLine (BlockEscape 12 13) (TextLine 13 18))
          (OrgBlockBodyLine (BlockEscape 18 19) (TextLine 19 25))
          (BlockEndLine 25 35)))))
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
          (OrgBlockBodyLine (TextLine 52 57))
          (BlockEndLine 57 67)))))
    (test-case "header keys stay source-backed across quotes and escapes"
      (let* ((source
              "#+HEADER: :var 'x :inner y' :results output\nsrc_sh[:var a\\ :inner :exports both]{echo hi}\n")
             (events (parse-org-rowan-events source))
             (keys
              (filter (lambda (event)
                        (and (eq? (car event) 'token)
                             (eq? (cadr event) 'SourceHeaderKey)))
                      events)))
        (check (org-events-cover-source? source events) => #t)
        (check (map (lambda (event)
                      (substring source (caddr event) (cadddr event)))
                    keys)
               => '("var" "results" "var" "exports"))))
    (test-case "dynamic-block parameters share the Scheme-owned header grammar"
      (check-org-ast-with parse-org-rowan-events
        "#+BEGIN: clocktable :scope file\n#+END:\n"
        (OrgFile
         (OrgDynamicBlock
          (BlockBeginLine 0 8) (DynamicBlockHeaderTrivia 8 9)
          (DynamicBlockName 9 19)
          (SourceHeaderTrivia 19 21) (SourceHeaderKey 21 26)
          (SourceHeaderTrivia 26 27) (SourceHeaderValue 27 31)
          (SourceHeaderTrivia 31 32) (BlockEndLine 32 39)))))
    (test-case "source switches are Scheme-classified before header arguments"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust -i -n 5 :exports both\nx\n#+end_src\n"
        (OrgFile
         (OrgSourceBlock
          (BlockBeginLine 0 11) (BlockHeaderTrivia 11 12)
          (SourceLanguage 12 16) (SourceHeaderTrivia 16 17)
          (SourceSwitchName 17 19) (SourceHeaderTrivia 19 20)
          (SourceSwitchName 20 22) (SourceHeaderTrivia 22 23)
          (SourceSwitchValue 23 24) (SourceHeaderTrivia 24 26)
          (SourceHeaderKey 26 33) (SourceHeaderTrivia 33 34)
          (SourceHeaderValue 34 38) (SourceHeaderTrivia 38 39)
          (OrgBlockBodyLine (TextLine 39 41))
          (BlockEndLine 41 51)))))
    (test-case "example switches use the same Scheme classification"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_example +n 12\nx\n#+end_example\n"
        (OrgFile
         (OrgExampleBlock
          (BlockBeginLine 0 15) (SourceHeaderTrivia 15 16)
          (SourceSwitchName 16 18) (SourceHeaderTrivia 18 19)
          (SourceSwitchValue 19 21) (SourceHeaderTrivia 21 22)
          (OrgBlockBodyLine (TextLine 22 24))
          (BlockEndLine 24 38)))))
    (test-case "optional switch argument does not consume a header key"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust -n :exports both\nx\n#+end_src\n"
        (OrgFile
         (OrgSourceBlock
          (BlockBeginLine 0 11) (BlockHeaderTrivia 11 12)
          (SourceLanguage 12 16) (SourceHeaderTrivia 16 17)
          (SourceSwitchName 17 19) (SourceHeaderTrivia 19 21)
          (SourceHeaderKey 21 28) (SourceHeaderTrivia 28 29)
          (SourceHeaderValue 29 33) (SourceHeaderTrivia 33 34)
          (OrgBlockBodyLine (TextLine 34 36))
          (BlockEndLine 36 46)))))
    (test-case "longer lookalike is not a declared switch"
      (check-org-ast-with parse-org-rowan-events
        "#+begin_src rust -invalid\nx\n#+end_src\n"
        (OrgFile
         (OrgSourceBlock
          (BlockBeginLine 0 11) (BlockHeaderTrivia 11 12)
          (SourceLanguage 12 16) (SourceHeaderTrivia 16 26)
          (OrgBlockBodyLine (TextLine 26 28))
          (BlockEndLine 28 38)))))
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
          (OrgBlockBodyLine (TextLine 31 36))
          (BlockEndLine 36 46)))))
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
          (OrgParagraph
           (OrgTextLine
            (TextLine 8 15)
            (OrgSubscript (InlineScriptDelimiter 15 16)
                          (InlineScriptValue 16 21))
            (TextLine 21 31)))
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
           (ListBullet 0 2)
           (OrgParagraph (OrgTextLine (TextLine 2 7)))
           (OrgComment (CommentLine 7 17) (CommentLine 17 26)))
          (OrgListItem
           (ListBullet 26 28)
           (OrgParagraph (OrgTextLine (TextLine 28 33))))))))
    (test-case "hash-prefixed text and keywords are not comments"
      (check-org-ast-with parse-org-rowan-events
        "#not-comment\n#+TITLE: Yes\n"
        (OrgFile
         (OrgParagraph (OrgTextLine (TextLine 0 13)))
         (OrgKeyword
          (KeywordTrivia 13 15) (KeywordKey 15 20)
          (KeywordTrivia 20 21)
          (OrgKeywordRawValue
           (KeywordTrivia 21 22)
           (OrgKeywordValue (OrgTextLine (TextLine 22 25))))
          (KeywordTrivia 25 26)))))
    (test-case "bare READONLY is a keyword but a longer marker remains prose"
      (check-org-ast-with parse-org-rowan-events
        "#+READONLY\n#+READONLYX\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 10)
          (OrgKeywordRawValue)
          (KeywordTrivia 10 11))
         (OrgParagraph (OrgTextLine (TextLine 11 23))))))
    (test-case "rich document keywords reuse Scheme inline Objects"
      (check-org-ast-with parse-org-rowan-events
        "#+TITLE: *Demo* Doc\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 7)
          (KeywordTrivia 7 8)
          (OrgKeywordRawValue
           (KeywordTrivia 8 9)
           (OrgKeywordValue
            (OrgTextLine
             (OrgBold (InlineMarkupDelimiter 9 10)
                      (InlineMarkupValue 10 14)
                      (InlineMarkupDelimiter 14 15))
             (TextLine 15 19))))
          (KeywordTrivia 19 20)))))
    (test-case "attribute keywords tokenize quoted values in Scheme"
      (check-org-ast-with parse-org-rowan-events
        "#+ATTR_HTML: :class compact :width \"10 em\"\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 11)
          (KeywordTrivia 11 12)
          (OrgKeywordRawValue
           (KeywordTrivia 12 13)
           (OrgKeywordAttributes
            (SourceHeaderTrivia 13 14)
            (SourceHeaderKey 14 19)
            (SourceHeaderTrivia 19 20)
            (SourceHeaderValue 20 27)
            (SourceHeaderTrivia 27 29)
            (SourceHeaderKey 29 34)
            (SourceHeaderTrivia 34 35)
            (SourceHeaderValue 35 42)))
          (KeywordTrivia 42 43)))))
    (test-case "INCLUDE path and options are source-backed Scheme events"
      (check-org-ast-with parse-org-rowan-events
        "#+INCLUDE: x.org\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 9) (KeywordTrivia 9 10)
          (OrgKeywordRawValue
           (KeywordTrivia 10 11)
           (OrgKeywordInclude
            (OrgIncludePath (IncludePathValue 11 16))
            (OrgIncludeTail)))
          (KeywordTrivia 16 17))))
      (check-org-ast-with parse-org-rowan-events
        "#+INCLUDE: x.org src org\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 9) (KeywordTrivia 9 10)
          (OrgKeywordRawValue
           (KeywordTrivia 10 11)
           (OrgKeywordInclude
            (OrgIncludePath (IncludePathValue 11 16))
            (OrgIncludeTail
             (IncludeTrivia 16 17) (IncludeArgument 17 20)
             (IncludeTrivia 20 21) (IncludeArgument 21 24))))
          (KeywordTrivia 24 25))))
      (check-org-ast-with parse-org-rowan-events
        "#+INCLUDE: \"./chapter one.org\" src org :lines \"1-20\" :minlevel 2 :only-contents\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 9) (KeywordTrivia 9 10)
          (OrgKeywordRawValue
           (KeywordTrivia 10 11)
           (OrgKeywordInclude
            (OrgIncludePath
             (IncludePathDelimiter 11 12) (IncludePathValue 12 29)
             (IncludePathDelimiter 29 30))
            (OrgIncludeTail
             (IncludeTrivia 30 31) (IncludeArgument 31 34)
             (IncludeTrivia 34 35) (IncludeArgument 35 38)
             (IncludeTrivia 38 39)
             (SourceHeaderTrivia 39 40) (SourceHeaderKey 40 45)
             (SourceHeaderTrivia 45 46) (SourceHeaderValue 46 52)
             (SourceHeaderTrivia 52 54) (SourceHeaderKey 54 62)
             (SourceHeaderTrivia 62 63) (SourceHeaderValue 63 64)
             (SourceHeaderTrivia 64 66) (SourceHeaderKey 66 79))))
          (KeywordTrivia 79 80))))
      (check-org-ast-with parse-org-rowan-events
        "#+INCLUDE: \"x.org\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 9) (KeywordTrivia 9 10)
          (OrgKeywordRawValue
           (KeywordTrivia 10 11)
           (OrgKeywordInclude
            (OrgIncludePath (IncludePathUnclosed 11 17))))
          (KeywordTrivia 17 18)))))
    (test-case "optional keyword hashes remain typed Scheme events"
      (check-org-ast-with parse-org-rowan-events
        "#+results[sha1]: prep-output\n"
        (OrgFile
         (OrgKeyword
          (KeywordTrivia 0 2) (KeywordKey 2 9)
          (KeywordTrivia 9 10) (KeywordOptional 10 14)
          (KeywordTrivia 14 16)
          (OrgKeywordRawValue (KeywordTrivia 16 17)
                              (KeywordValue 17 28))
          (KeywordTrivia 28 29)))))
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
                     (KeywordTrivia 10 11)
                     (OrgKeywordRawValue
                      (KeywordTrivia 11 12) (KeywordValue 12 23))
                     (KeywordTrivia 23 26))
         (OrgSection
          (OrgHeadline (HeadlineLine 26 27) (HeadlineTrivia 27 28)
                       (HeadlineTitle 28 37) (HeadlineTrivia 37 38))
          (OrgBabelCall (KeywordTrivia 38 40) (KeywordKey 40 44)
                        (KeywordTrivia 44 45)
                        (OrgKeywordRawValue
                         (KeywordTrivia 45 46) (KeywordValue 46 52))
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
    (test-case "property keys retain internal colons before their final delimiter"
      (check-org-ast-with parse-org-rowan-events
        "* H\n:PROPERTIES:\n:header-args:python: :session local\n:END:\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPropertyDrawer
           (DrawerBeginLine 4 17)
           (OrgNodeProperty
            (PropertyTrivia 17 18) (PropertyKey 18 36)
            (PropertyTrivia 36 38)
            (OrgSourceHeaderArgs
             (SourceHeaderTrivia 38 39) (SourceHeaderKey 39 46)
             (SourceHeaderTrivia 46 47) (SourceHeaderValue 47 52))
            (PropertyTrivia 52 53))
           (DrawerEndLine 53 59))))))
    (test-case "declared planning and clock keys retain headline context"
      (check-org-ast-with parse-org-rowan-events
        "* H\nSCHEDULED: now\nCLOCK: 2\n* N\nDEADLINE: x\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPlanning (PlanningKey 4 13) (PlanningTrivia 13 15)
                       (OrgPlanningValue (TextLine 15 18))
                       (PlanningTrivia 18 19))
          (OrgClock (ClockKey 19 24) (ClockTrivia 24 26)
                    (OrgClockValue (TextLine 26 27))
                    (ClockTrivia 27 28)))
         (OrgSection
          (OrgHeadline (HeadlineLine 28 29) (HeadlineTrivia 29 30)
                       (HeadlineTitle 30 31) (HeadlineTrivia 31 32))
          (OrgPlanning (PlanningKey 32 40) (PlanningTrivia 40 42)
                       (OrgPlanningValue (TextLine 42 43))
                       (PlanningTrivia 43 44))))))
    (test-case "one Planning Element keeps every declared key on its line"
      (check-org-ast-with parse-org-rowan-events
        "* H\nSCHEDULED: <a> DEADLINE: <b>\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPlanning
           (PlanningKey 4 13) (PlanningTrivia 13 15)
           (OrgPlanningValue (TextLine 15 18)) (PlanningTrivia 18 19)
           (PlanningKey 19 27) (PlanningTrivia 27 29)
           (OrgPlanningValue (TextLine 29 32)) (PlanningTrivia 32 33))))))
    (test-case "planning timestamps are Scheme-classified child Objects"
      (check-org-ast-with parse-org-rowan-events
        "* H\nSCHEDULED: <2026-05-15 Fri>\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgPlanning
           (PlanningKey 4 13) (PlanningTrivia 13 15)
           (OrgPlanningValue
            (OrgTimestampActive
             (TimestampDelimiter 15 16)
             (OrgTimestampPoint
              (TimestampDate 16 26)
              (TimestampTrivia 26 27)
              (TimestampDayName 27 30))
             (TimestampDelimiter 30 31)))
           (PlanningTrivia 31 32))))))
    (test-case "clock timestamp and duration are Scheme-classified fields"
      (check-org-ast-with parse-org-rowan-events
        "* H\nCLOCK: [2026-05-15 Fri 10:00] => 1:02\n"
        (OrgFile
         (OrgSection
          (OrgHeadline (HeadlineLine 0 1) (HeadlineTrivia 1 2)
                       (HeadlineTitle 2 3) (HeadlineTrivia 3 4))
          (OrgClock
           (ClockKey 4 9) (ClockTrivia 9 11)
           (OrgClockValue
            (OrgTimestampInactive
             (TimestampDelimiter 11 12)
             (OrgTimestampPoint
              (TimestampDate 12 22)
              (TimestampTrivia 22 23)
              (TimestampDayName 23 26)
              (TimestampTrivia 26 27)
              (TimestampTime 27 32))
             (TimestampDelimiter 32 33))
            (ClockTrivia 33 37)
            (ClockDuration 37 41))
           (ClockTrivia 41 42))))))
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
  ))
