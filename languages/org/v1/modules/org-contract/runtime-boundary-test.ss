;;; -*- Gerbil -*-
;;; This test deliberately imports neither gerbil-parser nor generator modules.

(import (only-in :std/test check test-case test-suite)
        (only-in :clan/poo/object .o .ref)
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
(export org-runtime-boundary-test)

(def org-runtime-boundary-test
  (test-suite "Org Contract runtime boundary"
    (test-case "graph manifest exposes source-owned fields"
      (let (keyword
            (car (filter (lambda (node)
                           (equal? (org-graph-node-label node) "keyword"))
                         org-v1-graph-shape)))
        (check (org-graph-node-rust (car org-v1-graph-shape)) => 'OrgFile)
        (check (org-graph-node-label (cadr org-v1-graph-shape))
               => "headline")
        (check (map org-graph-node-label
                    (filter (lambda (node)
                              (equal? (org-graph-node-rust node) 'OrgInlinetask))
                            org-v1-graph-shape))
               => '("inlinetask"))
        (check (map org-graph-field-label (org-graph-node-fields keyword))
               => '("key" "optional" "value" "value" "raw-value"))
        (check (map org-graph-field-mode (org-graph-node-fields keyword))
               => '(one one one node-text node-text))))
    (test-case "runtime Contract consumes POO Element graph without generator"
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
        (check (org-element-query? query) => #t)
        (check (org-contract-result-matched-count result) => 1)
        (check (org-contract-result-passed? result) => #t)))))
