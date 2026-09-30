;;; -*- Gerbil -*-
;;; Org-owned projection declarations shared by generator and runtime.

(import (only-in "modules/org-elements/graph-objects.ss"
                 make-org-graph-node make-org-graph-field
                 org-graph-node-rust org-graph-node-category
                 org-graph-node-label org-graph-node-fields
                 org-graph-field-rust org-graph-field-label
                 org-graph-field-mode))

(export org-v1-graph-shape org-v1-headline-extra-fields
        org-graph-node-rust org-graph-node-category
        org-graph-node-label org-graph-node-fields
        org-graph-field-rust org-graph-field-label org-graph-field-mode)

(def (field rust label (mode 'one))
  (make-org-graph-field rust label mode))

(def (node rust category label fields)
  (make-org-graph-node rust category label fields))

(def org-v1-headline-extra-fields
  '("source-title" "raw-value" "todo-keyword" "todo-type"
    "priority" "tags"))

(def timestamp-fields
  (list (field 'TimestampDelimiter "delimiter" 'each)
        (field 'TimestampRangeSeparator "range-separator" 'each)
        (field 'TimestampDate "date" 'each)
        (field 'TimestampDayName "day-name" 'each)
        (field 'TimestampTime "time" 'each)
        (field 'TimestampRepeater "repeater" 'each)
        (field 'TimestampDelay "delay" 'each)))

(def org-v1-graph-shape
  (list
   (node 'OrgFile "document" "org-data" '())
   (node 'OrgSection "section" "headline"
         (list (field 'HeadlineLine "markers")
               (field 'HeadlineTitle "title")
               (field 'OrgHeadlineTitle "title" 'node-text)
               (field 'HeadlineTitle "title-body")
               (field 'OrgHeadlineTitle "title-body" 'node-text)
               (field 'HeadlineTagValue "title")
               (field 'HeadlineTagTrivia "title")
               (field 'HeadlineTagValue "tag" 'each)))
   (node 'OrgInlinetask "element" "inlinetask"
         (list (field 'HeadlineLine "markers")
               (field 'HeadlineTitle "title")
               (field 'OrgHeadlineTitle "title" 'node-text)
               (field 'HeadlineTitle "title-body")
               (field 'OrgHeadlineTitle "title-body" 'node-text)
               (field 'HeadlineTagValue "title")
               (field 'HeadlineTagTrivia "title")
               (field 'HeadlineTagValue "tag" 'each)))
   (node 'OrgInlinetaskEnd "component" "inlinetask-end"
         (list (field 'HeadlineLine "markers")))
   (node 'OrgPropertyDrawer "element" "property-drawer" '())
   (node 'OrgDrawer "element" "drawer"
         (list (field 'DrawerName "name")))
   (node 'OrgParagraph "element" "paragraph" '())
   (node 'OrgFootnoteDefinition "element" "footnote-definition"
         (list (field 'FootnoteDefinitionLabel "label")))
   (node 'OrgComment "element" "comment"
         (list (field 'CommentLine "source-line" 'each)))
   (node 'OrgDiarySexp "element" "diary-sexp"
         (list (field 'DiarySexpValue "value")))
   (node 'OrgHorizontalRule "element" "horizontal-rule" '())
   (node 'OrgFixedWidth "element" "fixed-width"
         (list (field 'FixedWidthValue "value" 'each)))
   (node 'OrgKeyword "element" "keyword"
         (list (field 'KeywordKey "key")
               (field 'KeywordOptional "optional")
               (field 'KeywordValue "value")
               (field 'OrgKeywordValue "value" 'node-text)
               (field 'OrgSourceHeaderArgs "value" 'node-text)
               (field 'OrgKeywordAttributes "value" 'node-text)
               (field 'OrgKeywordInclude "value" 'node-text)
               (field 'OrgIncludePath "include-raw-path" 'node-text)
               (field 'IncludePathValue "include-path")
               (field 'IncludePathUnclosed "include-unclosed-path")
               (field 'IncludeArgument "include-argument" 'each)
               (field 'SourceHeaderKey "include-option-key" 'each)
               (field 'SourceHeaderValue "include-option-value" 'each)
               (field 'OrgKeywordValue "rich-value" 'node-text)
               (field 'SourceHeaderKey "attribute-key" 'each)
               (field 'SourceHeaderValue "attribute-value" 'each)
               (field 'SourceHeaderTrivia "header")
               (field 'SourceHeaderKey "header")
               (field 'SourceHeaderValue "header")
               (field 'SourceHeaderKey "header-key" 'each)
               (field 'SourceHeaderValue "header-value" 'each)
               (field 'OrgKeywordRawValue "raw-value" 'node-text)))
   (node 'OrgTagVocabulary "component" "tag-vocabulary"
         (list (field 'TagName "name" 'each)
               (field 'TagShortcut "shortcut" 'each)
               (field 'TagGroupSeparator "separator" 'each)))
   (node 'OrgTagExclusiveGroup "component" "tag-exclusive-group"
         (list (field 'TagName "name" 'each)
               (field 'TagShortcut "shortcut" 'each)
               (field 'TagGroupSeparator "separator" 'each)))
   (node 'OrgTagInclusiveGroup "component" "tag-inclusive-group"
         (list (field 'TagName "name" 'each)
               (field 'TagShortcut "shortcut" 'each)
               (field 'TagGroupSeparator "separator" 'each)))
   (node 'OrgBabelCall "element" "babel-call"
         (list (field 'KeywordKey "key")
               (field 'BabelCallName "name")
               (field 'BabelCallName "value")
               (field 'KeywordValue "value")
               (field 'OrgKeywordRawValue "raw-value" 'node-text)))
   (node 'OrgPlanning "element" "planning"
         (list (field 'PlanningKey "key" 'each)
               (field 'OrgPlanningValue "value" 'each-node-text)))
   (node 'OrgClock "element" "clock"
         (list (field 'OrgClockValue "value" 'node-text)
               (field 'ClockDuration "duration")))
   (node 'OrgPlainList "element" "plain-list" '())
   (node 'OrgListItem "element" "item"
         (list (field 'ListBullet "bullet")
               (field 'ListCounterValue "counter")
               (field 'ListCheckboxValue "checkbox")
               (field 'ListTagValue "tag")
               (field 'ListTrivia "trivia" 'each)))
   (node 'OrgTable "element" "table" '())
   (node 'OrgTableEl "element" "table-el" '())
   (node 'OrgTableRow "element" "table-row" '())
   (node 'OrgTableRuleRow "element" "table-rule-row" '())
   (node 'OrgTableCell "object" "table-cell"
         (list (field 'OrgTableCell "text" 'node-text)))
   (node 'OrgTableFormulaValue "object" "table-formula-value" '())
   (node 'OrgTableFormulaAssignment "object" "table-formula-assignment"
         (list (field 'FormulaFlag "flag" 'each)))
   (node 'OrgTableFormulaLhs "object" "table-formula-lhs" '())
   (node 'OrgTableFormulaRhs "object" "table-formula-rhs" '())
   (node 'OrgTableFormulaReference "object" "table-formula-reference"
         (list (field 'FormulaFieldReference "field")
               (field 'FormulaRowReference "row")
               (field 'FormulaRemoteReference "remote")))
   (node 'OrgNodeProperty "property" "node-property"
         (list (field 'PropertyKey "key")
               (field 'PropertyValue "value")
               (field 'OrgSourceHeaderArgs "value" 'node-text)
               (field 'SourceHeaderTrivia "header")
               (field 'SourceHeaderKey "header")
               (field 'SourceHeaderValue "header")
               (field 'SourceHeaderKey "header-key" 'each)
               (field 'SourceHeaderValue "header-value" 'each)))
   (node 'OrgSourceBlock "element" "src-block"
         (list (field 'SourceLanguage "language")
               (field 'BlockHeaderTrivia "header")
               (field 'SourceHeaderTrivia "header")
               (field 'SourceHeaderKey "header")
               (field 'SourceHeaderValue "header")
               (field 'SourceHeaderKey "header-key" 'each)
               (field 'SourceHeaderValue "header-value" 'each)
               (field 'SourceSwitchName "switch-name" 'each)
               (field 'SourceSwitchValue "switch-value" 'each)
               (field 'TextLine "body")
               (field 'OrgBlockBodyLine "raw-body" 'node-text)))
   (node 'OrgDynamicBlock "element" "dynamic-block"
         (list (field 'DynamicBlockName "name")
               (field 'BlockEndLine "end")
               (field 'DynamicBlockHeaderTrivia "header")
               (field 'SourceHeaderTrivia "header")
               (field 'SourceHeaderKey "header")
               (field 'SourceHeaderValue "header")
               (field 'SourceHeaderKey "header-key" 'each)
               (field 'SourceHeaderValue "header-value" 'each)))
   (node 'OrgSpecialBlock "element" "special-block"
         (list (field 'SpecialBlockName "name")
               (field 'BlockHeaderTrivia "header")))
   (node 'OrgLatexEnvironment "element" "latex-environment"
         (list (field 'LatexEnvironmentName "name")
               (field 'LatexEnvironmentBody "body" 'append-or-empty)))
   (node 'OrgQuoteBlock "element" "quote-block" '())
   (node 'OrgExampleBlock "element" "example-block"
         (list (field 'SourceHeaderTrivia "header")
               (field 'SourceSwitchName "switch-name" 'each)
               (field 'SourceSwitchValue "switch-value" 'each)
               (field 'TextLine "body")
               (field 'OrgBlockBodyLine "raw-body" 'node-text)))
   (node 'OrgVerseBlock "element" "verse-block" '())
   (node 'OrgCenterBlock "element" "center-block" '())
   (node 'OrgCommentBlock "element" "comment-block"
         (list (field 'TextLine "body")))
   (node 'OrgExportBlock "element" "export-block"
         (list (field 'ExportBackend "backend")
               (field 'TextLine "body")))
   (node 'OrgLink "object" "link"
         (list (field 'LinkTarget "path")
               (field 'LinkDescription "description")
               (field 'OrgLinkDescription "description" 'node-text)))
   (node 'OrgTarget "object" "target"
         (list (field 'InlineTargetValue "value")))
   (node 'OrgRadioTarget "object" "radio-target"
         (list (field 'InlineTargetValue "value")))
   (node 'OrgStatisticsCookie "object" "statistics-cookie"
         (list (field 'StatisticsCookieValue "value")))
   (node 'OrgLineBreak "object" "line-break" '())
   (node 'OrgExportSnippet "object" "export-snippet"
         (list (field 'ExportSnippetBackend "backend")
               (field 'ExportSnippetValue "value" 'append-or-empty)))
   (node 'OrgFootnoteReference "object" "footnote-reference"
         (list (field 'FootnoteReferenceLabel "label")
               (field 'FootnoteReferenceDefinition "definition")))
   (node 'OrgInlineSourceBlock "object" "inline-src-block"
         (list (field 'InlineSourceLanguage "language")
               (field 'InlineSourceParameters "parameters")
               (field 'OrgSourceHeaderArgs "parameters" 'node-text)
               (field 'SourceHeaderTrivia "header")
               (field 'SourceHeaderKey "header")
               (field 'SourceHeaderValue "header")
               (field 'SourceHeaderKey "header-key" 'each)
               (field 'SourceHeaderValue "header-value" 'each)
               (field 'InlineSourceBody "value" 'append-or-empty)))
   (node 'OrgInlineBabelCall "object" "inline-babel-call"
         (list (field 'InlineBabelCallName "call")
               (field 'InlineBabelInsideHeader "inside-header")
               (field 'InlineBabelArguments "arguments" 'append-or-empty)
               (field 'InlineBabelEndHeader "end-header")))
   (node 'OrgMacro "object" "macro"
         (list (field 'MacroName "name")
               (field 'MacroArguments "arguments" 'append-or-empty)))
   (node 'OrgCitation "object" "citation"
         (list (field 'CitationDelimiter "head")
               (field 'CitationGlobalPrefix "global-prefix" 'append-or-empty)
               (field 'OrgCitationGlobalPrefix "global-prefix" 'node-text)
               (field 'CitationGlobalSuffix "global-suffix" 'append-or-empty)
               (field 'OrgCitationGlobalSuffix "global-suffix" 'node-text)))
   (node 'OrgCitationReference "object" "citation-reference"
         (list (field 'CitationReferencePrefix "prefix" 'append-or-empty)
               (field 'OrgCitationReferencePrefix "prefix" 'node-text)
               (field 'CitationReferenceKey "key")
               (field 'CitationReferenceSuffix "suffix" 'append-or-empty)
               (field 'OrgCitationReferenceSuffix "suffix" 'node-text)))
   (node 'OrgCitationMalformedReference "object" "citation-malformed"
         (list (field 'CitationMalformedSegment "text")))
   (node 'OrgTimestampActive "object" "timestamp" timestamp-fields)
   (node 'OrgTimestampInactive "object" "timestamp" timestamp-fields)
   (node 'OrgTimestampDiary "object" "timestamp"
         (list (field 'TimestampDiaryExpression "diary-expression")
               (field 'TimestampTime "time" 'each)))
   (node 'OrgEntity "object" "entity"
         (list (field 'EntityName "name")
               (field 'EntityPost "post" 'append-or-empty)))
   (node 'OrgLaTeXFragment "object" "latex-fragment"
         (list (field 'LatexFragmentValue "value")))
   (node 'OrgCode "object" "code"
         (list (field 'InlineMarkupValue "value")))
   (node 'OrgVerbatim "object" "verbatim"
         (list (field 'InlineMarkupValue "value")))
   (node 'OrgBold "object" "bold"
         (list (field 'InlineMarkupValue "value")))
   (node 'OrgItalic "object" "italic"
         (list (field 'InlineMarkupValue "value")))
   (node 'OrgUnderline "object" "underline"
         (list (field 'InlineMarkupValue "value")))
   (node 'OrgSubscript "object" "subscript"
         (list (field 'InlineScriptValue "value")))
   (node 'OrgSuperscript "object" "superscript"
         (list (field 'InlineScriptValue "value")))
   (node 'OrgStrikeThrough "object" "strike-through"
         (list (field 'InlineMarkupValue "value")))
   (node 'OrgCloze "object" "cloze"
         (list (field 'ClozeText "text")
               (field 'ClozeHint "hint")
               (field 'ClozeId "id")))))
