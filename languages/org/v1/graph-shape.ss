;;; -*- Gerbil -*-
;;; Org-owned projection declarations shared by generator and runtime.

(export org-v1-graph-shape org-v1-headline-extra-fields
        org-graph-node-rust org-graph-node-category
        org-graph-node-label org-graph-node-fields
        org-graph-field-rust org-graph-field-label org-graph-field-mode)

(defstruct org-graph-node (rust category label fields))
(defstruct org-graph-field (rust label mode))

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
               (field 'ExportSnippetValue "value")))
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
   (node 'OrgStrikeThrough "object" "strike-through"
         (list (field 'InlineMarkupValue "value")))))
