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

(def org-v1-graph-shape
  (list
   (node 'OrgFile "document" "org-data" '())
   (node 'OrgSection "section" "headline"
         (list (field 'HeadlineLine "markers")
               (field 'HeadlineTitle "title")
               (field 'HeadlineTagValue "title")
               (field 'HeadlineTagTrivia "title")
               (field 'HeadlineTagValue "tag" 'each)))
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
   (node 'OrgFixedWidth "element" "fixed-width" '())
   (node 'OrgKeyword "element" "keyword"
         (list (field 'KeywordKey "key") (field 'KeywordValue "value")))
   (node 'OrgBabelCall "element" "babel-call"
         (list (field 'KeywordKey "key") (field 'KeywordValue "value")))
   (node 'OrgPlanning "element" "planning"
         (list (field 'PlanningKey "key" 'each)
               (field 'PlanningValue "value" 'each)))
   (node 'OrgClock "element" "clock"
         (list (field 'ClockValue "value")))
   (node 'OrgPlainList "element" "plain-list" '())
   (node 'OrgListItem "element" "item"
         (list (field 'ListBullet "bullet")
               (field 'ListCounterValue "counter")
               (field 'ListCheckboxValue "checkbox")
               (field 'ListTagValue "tag")
               (field 'ListTrivia "trivia" 'each)))
   (node 'OrgTable "element" "table" '())
   (node 'OrgTableRow "element" "table-row" '())
   (node 'OrgTableRuleRow "element" "table-rule-row" '())
   (node 'OrgTableCell "object" "table-cell"
         (list (field 'TableCellText "text")))
   (node 'OrgNodeProperty "property" "node-property"
         (list (field 'PropertyKey "key")
               (field 'PropertyValue "value")))
   (node 'OrgSourceBlock "element" "src-block"
         (list (field 'SourceLanguage "language")
               (field 'BlockHeaderTrivia "header")
               (field 'SourceHeaderTrivia "header")
               (field 'SourceHeaderKey "header")
               (field 'SourceHeaderValue "header")
               (field 'SourceHeaderKey "header-key" 'each)
               (field 'SourceHeaderValue "header-value" 'each)
               (field 'TextLine "body")))
   (node 'OrgDynamicBlock "element" "dynamic-block"
         (list (field 'DynamicBlockName "name")
               (field 'DynamicBlockHeaderTrivia "header")))
   (node 'OrgSpecialBlock "element" "special-block"
         (list (field 'SpecialBlockName "name")
               (field 'BlockHeaderTrivia "header")))
   (node 'OrgLatexEnvironment "element" "latex-environment"
         (list (field 'LatexEnvironmentName "name")
               (field 'LatexEnvironmentBody "body" 'append-or-empty)))
   (node 'OrgQuoteBlock "element" "quote-block" '())
   (node 'OrgExampleBlock "element" "example-block"
         (list (field 'TextLine "body")))
   (node 'OrgVerseBlock "element" "verse-block" '())
   (node 'OrgCenterBlock "element" "center-block" '())
   (node 'OrgCommentBlock "element" "comment-block"
         (list (field 'TextLine "body")))
   (node 'OrgExportBlock "element" "export-block"
         (list (field 'ExportBackend "backend")
               (field 'TextLine "body")))
   (node 'OrgLink "object" "link"
         (list (field 'LinkTarget "path")
               (field 'LinkDescription "description")))
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
         (list (field 'CitationGlobalPrefix "global-prefix" 'append-or-empty)
               (field 'CitationGlobalSuffix "global-suffix" 'append-or-empty)))
   (node 'OrgCitationReference "object" "citation-reference"
         (list (field 'CitationReferencePrefix "prefix" 'append-or-empty)
               (field 'CitationReferenceKey "key")
               (field 'CitationReferenceSuffix "suffix" 'append-or-empty)))
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
         (list (field 'InlineMarkupValue "value")))))
