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
               => '(OrgFile OrgSection OrgPropertyDrawer OrgDrawer OrgParagraph
                            OrgComment OrgDiarySexp
                            OrgHorizontalRule OrgFixedWidth
                            OrgKeyword OrgBabelCall OrgPlanning OrgClock
                            OrgPlainList OrgListItem
                            OrgTable OrgTableRow OrgTableRuleRow OrgTableCell
                            OrgNodeProperty OrgSourceBlock OrgDynamicBlock OrgQuoteBlock
                            OrgExampleBlock OrgVerseBlock OrgCenterBlock
                            OrgCommentBlock OrgExportBlock OrgLink
                            OrgTarget OrgRadioTarget OrgStatisticsCookie OrgLineBreak
                            OrgExportSnippet
                            OrgCode OrgVerbatim OrgBold OrgItalic OrgUnderline
                            OrgStrikeThrough))
        (check (map graph-node-label nodes)
               => '("org-data" "headline" "property-drawer" "drawer"
                                "paragraph" "comment" "diary-sexp"
                                "horizontal-rule" "fixed-width"
                                "keyword" "babel-call"
                                "planning" "clock"
                                "plain-list" "item" "table" "table-row"
                                "table-rule-row" "table-cell"
                                "node-property" "src-block" "dynamic-block" "quote-block"
                                "example-block" "verse-block" "center-block"
                                "comment-block" "export-block" "link"
                                "target" "radio-target" "statistics-cookie" "line-break"
                                "export-snippet"
                                "code" "verbatim" "bold" "italic"
                                "underline" "strike-through"))))
    (test-case "planning keeps each key and value independently"
      (let* ((nodes (graph-projection-nodes org-v1-graph-projection))
             (planning
              (car (filter (lambda (node)
                             (eq? (graph-node-syntax-kind node) 'OrgPlanning))
                           nodes)))
             (fields (graph-node-fields planning)))
        (check (map graph-field-name fields) => '("key" "value"))
        (check (map graph-field-mode fields) => '(each each))))
    (test-case "list item exposes Scheme-tokenized indentation and spacing"
      (let* ((nodes (graph-projection-nodes org-v1-graph-projection))
             (item (car (filter (lambda (node)
                                  (eq? (graph-node-syntax-kind node)
                                       'OrgListItem))
                                nodes)))
             (fields (graph-node-fields item)))
        (check (map graph-field-name fields)
               => '("bullet" "counter" "checkbox" "tag" "trivia"))
        (check (map graph-field-mode fields)
               => '(append append append append each))))
    (test-case "export snippet value remains present when its source span is empty"
      (let* ((nodes (graph-projection-nodes org-v1-graph-projection))
             (snippet (car (filter
                            (lambda (node)
                              (eq? (graph-node-syntax-kind node)
                                   'OrgExportSnippet))
                            nodes)))
             (fields (graph-node-fields snippet)))
        (check (map graph-field-name fields) => '("backend" "value"))
        (check (map graph-field-mode fields)
               => '(append append-or-empty))))))
