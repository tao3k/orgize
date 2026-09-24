;;; -*- Gerbil -*-
;;; This test deliberately imports neither gerbil-parser nor generator modules.

(import (only-in :clan/poo/object .o .ref)
        (only-in "../../graph-shape.ss"
                 org-v1-graph-shape org-graph-node-label
                 org-graph-node-rust org-graph-node-fields
                 org-graph-field-label org-graph-field-mode)
        (only-in "../org-elements/runtime-interface.ss"
                 make-org-element-graph-view make-org-element-query
                 org-element-query?)
        (only-in "runtime-interface.ss"
                 make-org-contract-assertion make-org-contract-expectation
                 org-contract-evaluate-assertion
                 org-contract-result-matched-count
                 org-contract-result-passed?))
(export run-org-runtime-boundary-test)

(def (assert-equal actual expected)
  (unless (equal? actual expected)
    (error "Org runtime boundary regression" actual expected)))

(def (run-org-runtime-boundary-test)
  (assert-equal (length org-v1-graph-shape) 24)
  (assert-equal (map org-graph-node-rust org-v1-graph-shape)
                '(OrgFile OrgSection OrgPropertyDrawer OrgDrawer OrgParagraph
                  OrgKeyword OrgBabelCall OrgPlanning OrgClock
                  OrgPlainList OrgListItem OrgTable OrgTableRow
                  OrgTableRuleRow OrgTableCell OrgNodeProperty
                  OrgSourceBlock OrgQuoteBlock OrgExampleBlock
                  OrgVerseBlock OrgCenterBlock OrgCommentBlock
                  OrgExportBlock OrgLink))
  (assert-equal (org-graph-node-label (cadr org-v1-graph-shape))
                "headline")
  (assert-equal (map org-graph-field-label
                     (org-graph-node-fields (list-ref org-v1-graph-shape 7)))
                '("key" "value"))
  (assert-equal (map org-graph-field-mode
                     (org-graph-node-fields (list-ref org-v1-graph-shape 7)))
                '(each each))
  (let* ((graph
          (make-org-element-graph-view
           (list (.o id: 0 parent: #f kind: "org-data" title: #f)
                 (.o id: 1 parent: 0 kind: "headline" title: "Evidence"))
           (lambda (record) (.ref record 'id))
           (lambda (record) (.ref record 'parent))
           (lambda (record) (.ref record 'kind))
           (lambda (record name) (.ref record (string->symbol name)))))
         (query (make-org-element-query "headline" "title" "Evidence"))
         (assertion
          (make-org-contract-assertion
           "evidence-title" 'error query
           (make-org-contract-expectation 'exactly 1)))
         (result (org-contract-evaluate-assertion assertion graph 0)))
    (assert-equal (org-element-query? query) #t)
    (assert-equal (org-contract-result-matched-count result) 1)
    (assert-equal (org-contract-result-passed? result) #t))
  #t)
