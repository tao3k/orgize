;;; -*- Gerbil -*-
;;; Org graph rules are source-owned POO declarations.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/list/list every)
        (only-in :clan/poo/object .o)
        (only-in :gerbil-parser/src/modules/parser/graph-projection-objects
                 graph-projection? graph-projection-nodes
                 graph-node-syntax-kind graph-node-label graph-node-fields
                 graph-field-name graph-field-mode)
        (only-in "modules/org-elements/graph-types.ss"
                 org-graph-node? org-graph-field?)
        (only-in "modules/org-elements/graph-objects.ss"
                 make-org-graph-node make-org-graph-field
                 org-graph-node-fields org-graph-field-mode)
        (only-in "graph-shape.ss" org-v1-graph-shape)
        (only-in "graph.ss" org-v1-graph-projection))
(export org-v1-graph-test)

(def org-v1-graph-test
  (test-suite "Org POO graph projection"
   (test-case "source graph shape is admitted as POO before projection"
     (let* ((field (make-org-graph-field 'HeadlineTitle "title" 'one))
            (node (make-org-graph-node 'OrgSection "section" "headline"
                                       (list field))))
       (check (org-graph-field? field) => #t)
       (check (org-graph-node? node) => #t)
       (check (every org-graph-node? org-v1-graph-shape) => #t)
       (check (org-graph-node-fields node) => (list field))
       (check (org-graph-field-mode field) => 'one)
       (check (org-graph-node? '(OrgSection section headline)) => #f)
       (check (org-graph-field?
               (.o kind: 'org-graph-field rust: 'HeadlineTitle
                   label: "title" mode: 'unknown))
              => #f)))
   (test-case "one declaration owns the contract scenario record kinds"
      (let (nodes (graph-projection-nodes org-v1-graph-projection))
        (check (graph-projection? org-v1-graph-projection) => #t)
        (check (map graph-node-syntax-kind nodes)
               => '(OrgFile OrgSection OrgPropertyDrawer OrgDrawer OrgParagraph
                            OrgFootnoteDefinition
                            OrgComment OrgDiarySexp
                            OrgHorizontalRule OrgFixedWidth
                            OrgKeyword OrgBabelCall OrgPlanning OrgClock
                            OrgPlainList OrgListItem
                            OrgTable OrgTableRow OrgTableRuleRow OrgTableCell
                            OrgNodeProperty OrgSourceBlock OrgDynamicBlock OrgSpecialBlock
                            OrgLatexEnvironment
                            OrgQuoteBlock
                            OrgExampleBlock OrgVerseBlock OrgCenterBlock
                            OrgCommentBlock OrgExportBlock OrgLink
                            OrgTarget OrgRadioTarget OrgStatisticsCookie OrgLineBreak
                            OrgExportSnippet OrgFootnoteReference
                            OrgInlineSourceBlock OrgInlineBabelCall OrgMacro OrgCitation
                            OrgCitationReference OrgEntity
                            OrgLaTeXFragment
                            OrgCode OrgVerbatim OrgBold OrgItalic OrgUnderline
                            OrgSubscript OrgSuperscript
                            OrgStrikeThrough))
        (check (map graph-node-label nodes)
               => '("org-data" "headline" "property-drawer" "drawer"
                                "paragraph" "footnote-definition" "comment" "diary-sexp"
                                "horizontal-rule" "fixed-width"
                                "keyword" "babel-call"
                                "planning" "clock"
                                "plain-list" "item" "table" "table-row"
                                "table-rule-row" "table-cell"
                                "node-property" "src-block" "dynamic-block" "special-block"
                                "latex-environment"
                                "quote-block"
                                "example-block" "verse-block" "center-block"
                                "comment-block" "export-block" "link"
                                "target" "radio-target" "statistics-cookie" "line-break"
                                "export-snippet" "footnote-reference"
                                "inline-src-block" "inline-babel-call" "macro" "citation"
                                "citation-reference" "entity"
                                "latex-fragment"
                                "code" "verbatim" "bold" "italic"
                                "underline" "subscript" "superscript"
                                "strike-through"))))
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
