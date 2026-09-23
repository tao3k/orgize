;;; -*- Gerbil -*-
;;; Contract behavior is Scheme/POO over Org Element facts, not S-expressions.

(import (only-in :std/test check test-case test-suite)
        (only-in :clan/poo/object .o .ref)
        (only-in "../org-elements/interface.ss"
                 make-org-element-query make-org-element-graph-view
                 make-org-element-query-context org-element-map
                 org-element-property org-element-lineage?
                 org-element-graph-records org-element-graph-id-of
                 org-element-graph-parent-of org-element-graph-kind-of
                 org-element-query? org-elements property child-of
                 descendant-of)
        (only-in "interface.ss"
                 make-org-contract-expectation
                 make-org-contract-binding make-org-contract-assertion
                 make-org-contract-definition org-contract-result-passed?
                 org-contract-result-matched-count
                 org-contract-evaluate-definition
                 org-contract-default-profile org-contract-profile?
                 assert-org-element))
(export org-contract-feature-test)

(def (fact id-value parent-value kind-value field-name-value field-value-value)
  (.o id: id-value parent: parent-value kind: kind-value
      field-name: field-name-value field-value: field-value-value))

(def sample-graph
  (make-org-element-graph-view
   (list (fact 0 #f "org-data" #f #f)
         (fact 1 0 "headline" "title" "Task")
         (fact 2 1 "headline" "title" "Evidence")
         (fact 3 2 "link" "path" "https://example.test")
         (fact 4 1 "node-property" "key" "CONTRACT_ORG"))
   (lambda (record) (.ref record 'id))
   (lambda (record) (.ref record 'parent))
   (lambda (record) (.ref record 'kind))
   (lambda (record name)
     (and (equal? name (.ref record 'field-name))
          (.ref record 'field-value)))))

(def org-contract-feature-test
  (test-suite "Org Contract POO feature"
    (test-case "query vocabulary is bound to declared Org Element fields"
      (check (org-contract-profile? org-contract-default-profile) => #t)
      (check (org-element-query? (make-org-element-query "headline" "title" "Task"))
             => #t)
      (check (org-element-query?
              (.o kind: 'org-element-query
                  schema: "orgize.org-elements.v1"
                  node-kind: "arbitrary-node"
                  field-name: #f field-value: #f
                  relation: 'any target: #f))
             => #f))
    (test-case "Org Element owns map, property, and lineage semantics"
      (let* ((context (make-org-element-query-context sample-graph))
             (headlines (org-element-map context "headline"
                                         (lambda (record) #t))))
        (check (length headlines) => 2)
        (check (org-element-property context (cadr headlines) "title")
               => "Evidence")
        (check (org-element-lineage? context 1 3) => #t)
        (check (org-element-lineage? context 2 1) => #f)))
    (test-case "bindings, ancestry, and counts evaluate over Org Element facts"
      (let* ((evidence
              (make-org-contract-binding
               "evidence"
               (make-org-element-query "headline" "title" "Evidence"
                                        'child-of 'scope)))
             (assertion
              (make-org-contract-assertion
               "section.has-evidence-link" 'error
               (make-org-element-query "link" #f #f
                                        'descendant-of "evidence")
               (make-org-contract-expectation 'at-least 1)
               (list evidence)))
             (contract
              (make-org-contract-definition "section.scope.v1" 'subtree
                                            (list assertion)))
             (results (org-contract-evaluate-definition
                       contract sample-graph 1)))
        (check (length results) => 1)
        (check (org-contract-result-matched-count (car results)) => 1)
        (check (org-contract-result-passed? (car results)) => #t)))
    (test-case "hygienic declaration lowers to admitted POO assertions"
      (let* ((assertions
              (list
               (assert-org-element "section.has-evidence-link" error
                 (bindings
                  (bind evidence
                    (org-elements headline (property title "Evidence")
                                  (child-of scope))))
                 (org-elements link (descendant-of evidence))
                 (expect at-least 1))))
             (contract
              (make-org-contract-definition "section.scope.v1" 'subtree
                                            assertions))
             (result (car (org-contract-evaluate-definition
                           contract sample-graph 1))))
        (check (org-contract-result-matched-count result) => 1)
        (check (org-contract-result-passed? result) => #t)))))
