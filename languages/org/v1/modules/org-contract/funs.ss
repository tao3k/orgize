;;; -*- Gerbil -*-
;;; Pure Org contract query and assertion algorithms over a typed graph view.

(import (only-in "types.ss" org-contract-definition?)
        (only-in "../org-elements/interface.ss"
                 org-element-graph-view? make-org-element-query-context
                 org-element-select org-element-query-target
                 org-element-graph-id-of)
        (only-in "objects.ss"
                 make-org-contract-result
                 org-contract-expectation-operator org-contract-expectation-count
                 org-contract-binding-name org-contract-binding-query
                 org-contract-assertion-id org-contract-assertion-bindings
                 org-contract-assertion-query org-contract-assertion-expectation
                 org-contract-definition-assertions))
(export org-contract-select org-contract-evaluate-assertion
        org-contract-evaluate-definition)

(def (target-ids target scope-id bindings)
  (cond
   ((eq? target 'scope) (list scope-id))
   ((string? target) (or (hash-get bindings target) '()))
   (else '())))

(def (select-with-context query context scope-id bindings)
  (org-element-select
   query context scope-id
   (target-ids (org-element-query-target query) scope-id bindings)))

(def (org-contract-select query graph scope-id (bindings (make-hash-table)))
  (unless (org-element-graph-view? graph)
    (error "Org contract query requires an admitted graph view" graph))
  (select-with-context query (make-org-element-query-context graph)
                       scope-id bindings))

(def (expectation-passed? expectation actual)
  (let (count (org-contract-expectation-count expectation))
    (case (org-contract-expectation-operator expectation)
      ((at-least) (>= actual count))
      ((exactly) (= actual count))
      ((at-most) (<= actual count))
      (else #f))))

(def (evaluate-assertion assertion graph context scope-id)
  (let ((bindings (make-hash-table))
        (id-of (org-element-graph-id-of graph)))
    (for-each
     (lambda (binding)
       (let (name (org-contract-binding-name binding))
         (when (hash-get bindings name)
           (error "duplicate Org contract binding" name))
         (hash-put!
          bindings name
          (map id-of
               (select-with-context (org-contract-binding-query binding)
                                    context scope-id bindings)))))
     (org-contract-assertion-bindings assertion))
    (let* ((matches
            (select-with-context (org-contract-assertion-query assertion)
                                 context scope-id bindings))
           (count (length matches)))
      (make-org-contract-result
       (org-contract-assertion-id assertion)
       count
       (expectation-passed? (org-contract-assertion-expectation assertion)
                            count)))))

(def (org-contract-evaluate-assertion assertion graph scope-id)
  (unless (org-element-graph-view? graph)
    (error "Org contract evaluation requires an admitted graph view" graph))
  (evaluate-assertion assertion graph
                      (make-org-element-query-context graph) scope-id))

(def (org-contract-evaluate-definition definition graph scope-id)
  (unless (and (org-contract-definition? definition)
               (org-element-graph-view? graph))
    (error "Org contract evaluation requires admitted POO values"))
  (let (context (make-org-element-query-context graph))
    (map (lambda (assertion)
           (evaluate-assertion assertion graph context scope-id))
         (org-contract-definition-assertions definition))))
