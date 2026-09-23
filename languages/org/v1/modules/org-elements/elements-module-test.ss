;;; -*- Gerbil -*-
;;; Org Elements is a POO feature independently of Org Contract.

(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :clan/poo/object .o .ref)
        (only-in "test-syntax.ss"
                 check-org-element-catalog check-org-element-selection)
        (only-in "interface.ss"
                 +org-element-kinds+ org-elements-default-profile
                 make-org-element-graph-view make-org-element-query
                 make-org-element-query-context org-element-query?
                 org-element-map org-element-property
                 org-element-lineage? org-element-select
                 org-elements property property-contains at child-of))
(export org-elements-module-test)

(def sample-graph
  (make-org-element-graph-view
   (list (.o id: 0 parent: #f kind: "org-data" title: #f)
         (.o id: 1 parent: 0 kind: "headline" title: "Evidence")
         (.o id: 2 parent: 1 kind: "link" title: #f))
   (lambda (record) (.ref record 'id))
   (lambda (record) (.ref record 'parent))
   (lambda (record) (.ref record 'kind))
   (lambda (record name)
     (and (equal? name "title") (.ref record 'title)))))

(def org-elements-module-test
  (test-suite "Org Elements POO module"
    (test-case "catalog and projected query share one feature interface"
      (check-org-element-catalog)
      (check (if (member "headline" +org-element-kinds+) #t #f)
             => #t)
      (check (.ref org-elements-default-profile 'kind) =>
             'org-elements-profile)
      (check (org-element-query? (make-org-element-query "headline" "title" "Evidence"))
             => #t))
    (test-case "map, property, lineage, and scoped selection compose"
      (let* ((context (make-org-element-query-context sample-graph))
             (headlines (org-element-map context "headline"
                                         (lambda (record) #t)))
             (query (org-elements headline (property title "Evidence")
                                  (child-of scope)))
             (reordered (org-elements headline (child-of scope)
                                      (property title "Evidence"))))
        (check (length headlines) => 1)
        (check (org-element-property context (car headlines) "title")
               => "Evidence")
        (check (org-element-lineage? context 0 2) => #t)
        (check (org-element-query? reordered) => #t)
        (check-org-element-selection
         query context 0 (list 0)
         (lambda (record) (.ref record 'id)) (list 1))
        (check-org-element-selection
         reordered context 0 (list 0)
         (lambda (record) (.ref record 'id)) (list 1))
        (check-org-element-selection
         (org-elements headline (property-contains title "Evid")
                       (at scope))
         context 1 (list 1)
         (lambda (record) (.ref record 'id)) (list 1))))
    (test-case "query syntax rejects conflicting and unknown declarations"
      (check-exception
       (org-elements headline (property title "A")
                     (property title "B"))
       true)
      (check-exception (org-elements invented-kind) true))))
