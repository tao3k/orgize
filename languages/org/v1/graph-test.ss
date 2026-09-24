;;; -*- Gerbil -*-
;;; Org graph rules are source-owned POO declarations.

(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/src/modules/parser/graph-projection-objects
                 graph-projection? graph-projection-nodes
                 graph-node-syntax-kind graph-node-label graph-node-fields
                 graph-field-name graph-field-mode)
        (only-in "graph.ss" org-v1-graph-projection))
(export org-v1-graph-test)

(def org-v1-graph-test
  (test-suite "Org POO graph projection"
    (test-case "one declaration owns the contract scenario record kinds"
      (let (nodes (graph-projection-nodes org-v1-graph-projection))
        (check (graph-projection? org-v1-graph-projection) => #t)
        (check (map graph-node-syntax-kind nodes)
               => '(OrgFile OrgSection OrgPropertyDrawer OrgParagraph
                            OrgKeyword OrgBabelCall OrgPlanning OrgClock
                            OrgPlainList OrgListItem
                            OrgTable OrgTableRow OrgTableRuleRow OrgTableCell
                            OrgNodeProperty OrgSourceBlock OrgQuoteBlock
                            OrgExampleBlock OrgVerseBlock OrgCenterBlock
                            OrgCommentBlock OrgExportBlock OrgLink))
        (check (map graph-node-label nodes)
               => '("org-data" "headline" "property-drawer"
                                "paragraph" "keyword" "babel-call" "planning" "clock"
                                "plain-list" "item" "table" "table-row"
                                "table-rule-row" "table-cell"
                                "node-property" "src-block" "quote-block"
                                "example-block" "verse-block" "center-block"
                                "comment-block" "export-block" "link"))))
    (test-case "planning keeps each key and value independently"
      (let* ((nodes (graph-projection-nodes org-v1-graph-projection))
             (planning
              (car (filter (lambda (node)
                             (eq? (graph-node-syntax-kind node) 'OrgPlanning))
                           nodes)))
             (fields (graph-node-fields planning)))
        (check (map graph-field-name fields) => '("key" "value"))
        (check (map graph-field-mode fields) => '(each each))))))
