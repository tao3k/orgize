;;; -*- Gerbil -*-
;;; POO admission for projected Org Element queries and host graph views.

(import (only-in :clan/poo/object .ref .slot? object?)
        (only-in :clan/poo/mop define-type Type. element?)
        (only-in :std/list/list every)
        (only-in :gerbil-parser/src/modules/parser/graph-projection-objects
                 graph-projection-nodes graph-node-label graph-node-fields
                 graph-field-name)
        (only-in "../../graph.ss" org-v1-graph-projection))
(export +org-element-schema+ +org-element-query-kind+
        +org-element-clause-kind+ +org-element-graph-kind+
        +org-element-context-kind+ +org-element-profile-kind+
        +org-element-named-query-kind+
        OrgElementQuery OrgElementQueryClause
        OrgNamedElementQuery org-named-element-query?
        OrgElementGraphView OrgElementQueryContext OrgElementsProfile
        org-element-query? org-element-query-clause?
        org-element-graph-view?
        org-element-query-context? org-elements-profile?)

(def +org-element-schema+ "orgize.org-elements.v1")
(def +org-element-query-kind+ 'org-element-query)
(def +org-element-clause-kind+ 'org-element-query-clause)
(def +org-element-graph-kind+ 'org-element-graph-view)
(def +org-element-context-kind+ 'org-element-query-context)
(def +org-element-profile-kind+ 'org-elements-profile)
(def +org-element-named-query-kind+ 'org-named-element-query)

(def (has-kind-and-slots? value kind slots)
  (and (object? value) (.slot? value 'kind)
       (eq? (.ref value 'kind) kind)
       (every (lambda (slot) (.slot? value slot)) slots)))

(def (nonempty-string? value)
  (and (string? value) (> (string-length value) 0)))

(def (org-element-kind-rule label)
  (let loop ((rules (graph-projection-nodes org-v1-graph-projection)))
    (cond
     ((null? rules) #f)
     ((equal? label (graph-node-label (car rules))) (car rules))
     (else (loop (cdr rules))))))

(def (org-element-field? rule name)
  (and rule
       (or (and (equal? (graph-node-label rule) "headline")
                (member name '("source-title" "raw-value"
                               "todo-keyword" "todo-type"
                               "priority" "tags")))
           (let loop ((fields (graph-node-fields rule)))
             (cond
              ((null? fields) #f)
              ((equal? name (graph-field-name (car fields))) #t)
              (else (loop (cdr fields))))))))

(def (org-element-query-shape? value)
  (and (has-kind-and-slots?
        value +org-element-query-kind+
        '(schema node-kind field-name field-value field-match relation target))
       (equal? (.ref value 'schema) +org-element-schema+)
       (let (rule (org-element-kind-rule (.ref value 'node-kind)))
         (and rule
              (let (field-name (.ref value 'field-name))
                (or (not field-name)
                    (org-element-field? rule field-name)))))
       (let ((field-name (.ref value 'field-name))
             (field-value (.ref value 'field-value)))
         (or (and (not field-name) (not field-value))
             (and (nonempty-string? field-name) (string? field-value))))
       (memq (.ref value 'field-match) '(exact contains))
       (memq (.ref value 'relation) '(any at child-of descendant-of))
       (let (target (.ref value 'target))
         (if (eq? (.ref value 'relation) 'any)
           (not target)
           (or (eq? target 'scope) (nonempty-string? target))))))

(define-type (OrgElementQuery @ Type.)
  .element?: org-element-query-shape?)

(def (org-element-query-clause-shape? value)
  (and (has-kind-and-slots? value +org-element-clause-kind+
                            '(schema clause-kind name value match))
       (equal? (.ref value 'schema) +org-element-schema+)
       (case (.ref value 'clause-kind)
         ((property)
          (and (nonempty-string? (.ref value 'name))
               (string? (.ref value 'value))
               (memq (.ref value 'match) '(exact contains))))
         ((relation)
          (and (memq (.ref value 'name) '(at child-of descendant-of))
               (not (.ref value 'match))
               (or (eq? (.ref value 'value) 'scope)
                   (nonempty-string? (.ref value 'value)))))
         (else #f))))

(define-type (OrgElementQueryClause @ Type.)
  .element?: org-element-query-clause-shape?)

(def (org-element-graph-view-shape? value)
  (and (has-kind-and-slots?
        value +org-element-graph-kind+
        '(schema records id-of parent-of kind-of field-of))
       (equal? (.ref value 'schema) +org-element-schema+)
       (list? (.ref value 'records))
       (every procedure?
              (map (lambda (slot) (.ref value slot))
                   '(id-of parent-of kind-of field-of)))))

(define-type (OrgElementGraphView @ Type.)
  .element?: org-element-graph-view-shape?)

(def (org-element-context-shape? value)
  (and (has-kind-and-slots? value +org-element-context-kind+
                            '(schema graph index))
       (equal? (.ref value 'schema) +org-element-schema+)
       (element? OrgElementGraphView (.ref value 'graph))))

(define-type (OrgElementQueryContext @ Type.)
  .element?: org-element-context-shape?)

(def (org-elements-profile-shape? value)
  (and (has-kind-and-slots? value +org-element-profile-kind+
                            '(schema inventory projection))
       (equal? (.ref value 'schema) +org-element-schema+)
       (list? (.ref value 'inventory))))

(define-type (OrgElementsProfile @ Type.)
  .element?: org-elements-profile-shape?)

(def (org-element-query? value) (element? OrgElementQuery value))
(def (org-named-element-query-shape? value)
  (and (has-kind-and-slots? value +org-element-named-query-kind+
                            '(schema id query))
       (equal? (.ref value 'schema) +org-element-schema+)
       (nonempty-string? (.ref value 'id))
       (org-element-query? (.ref value 'query))))

(define-type (OrgNamedElementQuery @ Type.)
  .element?: org-named-element-query-shape?)

(def (org-named-element-query? value)
  (element? OrgNamedElementQuery value))
(def (org-element-query-clause? value)
  (element? OrgElementQueryClause value))
(def (org-element-graph-view? value)
  (element? OrgElementGraphView value))
(def (org-element-query-context? value)
  (element? OrgElementQueryContext value))
(def (org-elements-profile? value)
  (element? OrgElementsProfile value))
