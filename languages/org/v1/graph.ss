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
    (make-graph-node
     'OrgLink "object" "link"
     (list (make-graph-field 'LinkTarget "path")
           (make-graph-field 'LinkDescription "description"))))))
