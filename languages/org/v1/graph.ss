;;; -*- Gerbil -*-
;;; Org-owned graph projection: no Org syntax identities in the Rowan engine.

(import (only-in :gerbil-parser/graph-projection-support
                 make-graph-projection make-graph-node make-graph-field))
(export org-v1-graph-projection)

(def org-v1-graph-projection
  (make-graph-projection
   (list
    (make-graph-node 'OrgFile "document" "org-data" '())
    (make-graph-node
     'OrgSection "section" "headline"
     (list (make-graph-field 'HeadlineLine "markers")
           (make-graph-field 'HeadlineTitle "title")))
    (make-graph-node 'OrgPropertyDrawer "element" "property-drawer" '())
    (make-graph-node 'OrgParagraph "element" "paragraph" '())
    (make-graph-node 'OrgKeyword "element" "keyword"
                     (list (make-graph-field 'KeywordKey "key")
                           (make-graph-field 'KeywordValue "value")))
    (make-graph-node 'OrgBabelCall "element" "babel-call"
                     (list (make-graph-field 'KeywordKey "key")
                           (make-graph-field 'KeywordValue "value")))
    (make-graph-node
     'OrgPlanning "element" "planning"
     (list (make-graph-field 'PlanningKey "key" 'each)
           (make-graph-field 'PlanningValue "value" 'each)))
    (make-graph-node
     'OrgClock "element" "clock"
     (list (make-graph-field 'ClockValue "value")))
    (make-graph-node 'OrgPlainList "element" "plain-list" '())
    (make-graph-node 'OrgListItem "element" "item"
                     (list (make-graph-field 'ListBullet "bullet")))
    (make-graph-node 'OrgTable "element" "table" '())
    (make-graph-node 'OrgTableRow "element" "table-row" '())
    (make-graph-node 'OrgTableRuleRow "element" "table-rule-row" '())
    (make-graph-node 'OrgTableCell "object" "table-cell"
                     (list (make-graph-field 'TableCellText "text")))
    (make-graph-node
     'OrgNodeProperty "property" "node-property"
     (list (make-graph-field 'PropertyKey "key")
           (make-graph-field 'PropertyValue "value")))
    (make-graph-node
     'OrgSourceBlock "element" "src-block"
     (list (make-graph-field 'SourceLanguage "language")
           (make-graph-field 'BlockHeaderTrivia "header")
           (make-graph-field 'TextLine "body")))
    (make-graph-node 'OrgQuoteBlock "element" "quote-block" '())
    (make-graph-node 'OrgExampleBlock "element" "example-block"
                     (list (make-graph-field 'TextLine "body")))
    (make-graph-node 'OrgVerseBlock "element" "verse-block" '())
    (make-graph-node 'OrgCenterBlock "element" "center-block" '())
    (make-graph-node 'OrgCommentBlock "element" "comment-block"
                     (list (make-graph-field 'TextLine "body")))
    (make-graph-node
     'OrgExportBlock "element" "export-block"
     (list (make-graph-field 'ExportBackend "backend")
           (make-graph-field 'TextLine "body")))
    (make-graph-node
     'OrgLink "object" "link"
     (list (make-graph-field 'LinkTarget "path")
           (make-graph-field 'LinkDescription "description"))))))
