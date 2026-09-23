;;; -*- Gerbil -*-
;;; Org graph rules are source-owned POO declarations.

(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/src/modules/parser/graph-projection-objects
                 graph-projection? graph-projection-nodes
                 graph-node-syntax-kind graph-node-label)
        (only-in "graph.ss" org-v1-graph-projection))
(export org-v1-graph-test)

(def org-v1-graph-test
  (test-suite "Org POO graph projection"
    (test-case "one declaration owns the contract scenario record kinds"
      (let (nodes (graph-projection-nodes org-v1-graph-projection))
        (check (graph-projection? org-v1-graph-projection) => #t)
        (check (map graph-node-syntax-kind nodes)
               => '(OrgFile OrgSection OrgPropertyDrawer OrgParagraph
                            OrgTable OrgTableRow OrgTableRuleRow OrgTableCell
                            OrgNodeProperty OrgSourceBlock OrgLink))
        (check (map graph-node-label nodes)
               => '("org-data" "headline" "property-drawer"
                                "paragraph" "table" "table-row"
                                "table-rule-row" "table-cell"
                                "node-property" "src-block" "link"))))))
