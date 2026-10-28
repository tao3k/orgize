;;; -*- Gerbil -*-
;;; Org graph rules are source-owned POO declarations.

(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :std/list/list every)
        (only-in :clan/poo/object .o)
        (only-in :gerbil-parser/graph-projection-support
                 make-graph-node make-graph-field make-graph-projection graph-projection?)
        (only-in :gerbil-parser/src/graph/graph-projection-objects
                 graph-projection-nodes graph-node-syntax-kind graph-node-label graph-node-fields
                 graph-field-name graph-field-mode)
        (only-in "../languages/org/modules/org-elements/graph-shape.ss" org-graph-shape)
        (only-in "../languages/org/modules/org-elements/graph.ss" org-graph-projection))
(export org-graph-test)

(def org-graph-test
  (test-suite "Org POO graph projection"
   (test-case "source graph shape uses Parser's admitted object family directly"
     (let* ((field (make-graph-field 'HeadlineTitle "title"))
            (node (make-graph-node 'OrgSection "section" "headline" (list field))))
       (check (graph-projection? (make-graph-projection (list node))) => #t)
       (check (graph-projection? (make-graph-projection org-graph-shape)) => #t)
       (check (graph-node-fields node) => (list field))
       (check (graph-field-mode field) => 'append)
       (check-exception (make-graph-field 'HeadlineTitle "title" 'unknown) exception?)))
   (test-case "one declaration owns the contract scenario record kinds"
      (let (nodes (graph-projection-nodes org-graph-projection))
        (check (graph-projection? org-graph-projection) => #t)
      (check (map graph-node-syntax-kind nodes)
               => '(OrgFile OrgSection OrgInlinetask OrgInlinetaskEnd
                            OrgPropertyDrawer OrgDrawer OrgParagraph
                            OrgFootnoteDefinition
                            OrgComment OrgDiarySexp
                            OrgHorizontalRule OrgFixedWidth
                            OrgKeyword OrgTagVocabulary OrgTagExclusiveGroup
                            OrgTagInclusiveGroup OrgBabelCall OrgPlanning OrgClock
                            OrgPlainList OrgListItem
                            OrgTable OrgTableEl OrgTableRow OrgTableRuleRow OrgTableCell
                            OrgTableFormulaValue OrgTableFormulaAssignment
                            OrgTableFormulaLhs OrgTableFormulaRhs OrgTableFormulaReference
                            OrgNodeProperty OrgSourceBlock OrgDynamicBlock OrgSpecialBlock
                            OrgLatexEnvironment
                            OrgQuoteBlock
                            OrgExampleBlock OrgVerseBlock OrgCenterBlock
                            OrgCommentBlock OrgExportBlock OrgLink
                            OrgTarget OrgRadioTarget OrgStatisticsCookie OrgLineBreak
                            OrgExportSnippet OrgFootnoteReference
                            OrgInlineSourceBlock OrgInlineBabelCall OrgMacro OrgMacroArgument OrgCitation
                            OrgCitationReference OrgCitationMalformedReference
                            OrgTimestampActive OrgTimestampInactive OrgTimestampDiary
                            OrgEntity
                            OrgLaTeXFragment
                            OrgCode OrgVerbatim OrgBold OrgItalic OrgUnderline
                            OrgSubscript OrgSuperscript
                            OrgStrikeThrough OrgCloze))
        (check (map graph-node-label nodes)
               => '("org-data" "headline" "inlinetask" "inlinetask-end"
                                "property-drawer" "drawer"
                                "paragraph" "footnote-definition" "comment" "diary-sexp"
                                "horizontal-rule" "fixed-width"
                                "keyword" "tag-vocabulary" "tag-exclusive-group"
                                "tag-inclusive-group" "babel-call"
                                "planning" "clock"
                                "plain-list" "item" "table" "table-el" "table-row"
                                "table-rule-row" "table-cell"
                                "table-formula-value" "table-formula-assignment"
                                "table-formula-lhs" "table-formula-rhs"
                                "table-formula-reference"
                                "node-property" "src-block" "dynamic-block" "special-block"
                                "latex-environment"
                                "quote-block"
                                "example-block" "verse-block" "center-block"
                                "comment-block" "export-block" "link"
                                "target" "radio-target" "statistics-cookie" "line-break"
                                "export-snippet" "footnote-reference"
                                "inline-src-block" "inline-babel-call" "macro" "macro-argument" "citation"
                                "citation-reference" "citation-malformed"
                                "timestamp" "timestamp"
                                "timestamp" "entity"
                                "latex-fragment"
                                "code" "verbatim" "bold" "italic"
                                "underline" "subscript" "superscript"
                                "strike-through" "cloze"))))
    (test-case "dynamic block exposes its Scheme closing line"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (dynamic (car (filter (lambda (node)
                                     (eq? (graph-node-syntax-kind node)
                                          'OrgDynamicBlock))
                                   nodes)))
             (end-fields (filter (lambda (field)
                                   (equal? (graph-field-name field) "end"))
                                 (graph-node-fields dynamic))))
        (check (map graph-field-mode end-fields) => '(append))))
    (test-case "drawer exposes its Scheme body boundaries"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (drawer (car (filter (lambda (node)
                                   (eq? (graph-node-syntax-kind node) 'OrgDrawer))
                                 nodes)))
             (fields (graph-node-fields drawer)))
        (check (map graph-field-name fields)
               => '("name" "header-trivia" "end"))))
    (test-case "planning keeps each key and value independently"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (planning
              (car (filter (lambda (node)
                             (eq? (graph-node-syntax-kind node) 'OrgPlanning))
                           nodes)))
             (fields (graph-node-fields planning)))
        (check (map graph-field-name fields) => '("key" "value"))
        (check (map graph-field-mode fields) => '(each each-node-text))))
    (test-case "clock value and duration are source-backed graph fields"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (clock (car (filter (lambda (node)
                                   (eq? (graph-node-syntax-kind node) 'OrgClock))
                                 nodes)))
             (fields (graph-node-fields clock)))
        (check (map graph-field-name fields) => '("value" "duration"))
        (check (map graph-field-mode fields) => '(node-text append))))
    (test-case "list item exposes Scheme-tokenized indentation and spacing"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (item (car (filter (lambda (node)
                                  (eq? (graph-node-syntax-kind node)
                                       'OrgListItem))
                                nodes)))
             (fields (graph-node-fields item)))
        (check (map graph-field-name fields)
               => '("bullet" "counter" "checkbox" "tag" "trivia"))
        (check (map graph-field-mode fields)
               => '(append append append append each))))
    (test-case "timestamp graph fields preserve source-backed query properties"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (timestamps
              (filter (lambda (node)
                        (memq (graph-node-syntax-kind node)
                              '(OrgTimestampActive OrgTimestampInactive)))
                      nodes)))
        (check (length timestamps) => 2)
        (check (map graph-field-name
                    (graph-node-fields (car timestamps)))
               => '("delimiter" "range-separator" "time-range-separator"
                    "point" "date" "year" "year" "month" "month" "day" "day"
                    "second-year" "second-month" "second-day"
                    "day-name" "day-name" "second-day-name"
                    "time" "first-time" "time" "second-time" "inline-end-time"
                    "hour" "minute" "inline-end-hour" "inline-end-minute"
                    "second-hour" "second-minute"
                    "repeater" "repeater-mark" "repeater-value" "repeater-unit"
                    "delay" "delay-mark" "delay-value" "delay-unit"))
        (check (map graph-field-mode
                    (graph-node-fields (cadr timestamps)))
               => '(each each each each-node-text each-node-text
                    each each each each each each append append append
                    each each append each-node-text node-text each-node-text
                    node-text node-text each each append append append append
                    each-node-text each each each each-node-text each each each))))
    (test-case "export snippet value remains present when its source span is empty"
      (let* ((nodes (graph-projection-nodes org-graph-projection))
             (snippet (car (filter
                            (lambda (node)
                              (eq? (graph-node-syntax-kind node)
                                   'OrgExportSnippet))
                            nodes)))
             (fields (graph-node-fields snippet)))
        (check (map graph-field-name fields) => '("backend" "value"))
        (check (map graph-field-mode fields)
               => '(append append-or-empty))))))
