;;; -*- Gerbil -*-
;;; Pure Org Element map, property, lineage, and scoped selection.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop element?)
        (only-in :gerbil-parser/graph-query-support
                 make-graph-query-context
                 graph-query-map graph-query-lineage? graph-query-select)
        (only-in "types.ss"
                 +org-element-schema+ +org-element-context-kind+
                 OrgElementQueryContext org-element-query?
                 org-element-query-clause? org-element-predicate?
                 org-element-graph-view? org-element-query-context?)
        (only-in "objects.ss"
                 make-org-element-query-groups org-element-clause-kind
                 make-org-element-predicate
                 org-element-clause-name org-element-clause-value
                 org-element-clause-match
                 org-element-query-node-kind org-element-query-groups
                 org-element-predicate-groups
                 org-element-query-relation
                 org-element-graph-field-of))
(export org-element-query-compose org-element-predicate-all
        org-element-predicate-any
        make-org-element-query-context org-element-map
        org-element-property org-element-lineage? org-element-select)

(def (conjoin-groups left right)
  (let (combined
        (apply append
               (map (lambda (group)
                      (map (lambda (other) (append group other)) right))
                    left)))
    (when (or (> (length combined) 32)
              (ormap (lambda (group) (> (length group) 16)) combined))
      (error "Org Element predicate expansion exceeds the AOT bound"))
    combined))

(def (predicate-clause-groups clause)
  (cond
   ((org-element-predicate? clause) (org-element-predicate-groups clause))
   ((and (org-element-query-clause? clause)
         (eq? (org-element-clause-kind clause) 'property))
    (list (list clause)))
   (else (error "Org Element predicate requires POO property clauses" clause))))

(def (org-element-predicate-all clauses)
  (unless (pair? clauses)
    (error "all-of requires at least one Org Element predicate"))
  (make-org-element-predicate
   (let loop ((rest clauses) (groups (list '())))
     (if (null? rest) groups
       (loop (cdr rest)
             (conjoin-groups groups
                             (predicate-clause-groups (car rest))))))))

(def (org-element-predicate-any clauses)
  (unless (pair? clauses)
    (error "any-of requires at least one Org Element predicate"))
  (let (groups (apply append (map predicate-clause-groups clauses)))
    (when (> (length groups) 32)
      (error "Org Element predicate alternatives exceed the AOT bound"))
    (make-org-element-predicate groups)))

(def (org-element-query-compose kind clauses)
  (let loop ((rest clauses) (groups (list '()))
             (relation 'any) (target #f))
    (if (null? rest)
      (make-org-element-query-groups kind groups relation target)
      (let (clause (car rest))
        (if (and (org-element-query-clause? clause)
                 (eq? (org-element-clause-kind clause) 'relation))
          (begin
           (unless (eq? relation 'any)
             (error "duplicate Org Element relation clause" kind))
           (loop (cdr rest) groups
                 (org-element-clause-name clause)
                 (org-element-clause-value clause)))
          (loop (cdr rest)
                (conjoin-groups groups (predicate-clause-groups clause))
                relation target))))))

(def (make-org-element-query-context graph-value)
  (unless (org-element-graph-view? graph-value)
    (error "Org Element query requires an admitted graph view" graph-value))
  (let (value
        (.o kind: +org-element-context-kind+
            schema: +org-element-schema+
            graph: graph-value
            index: (make-graph-query-context graph-value)))
    (unless (element? OrgElementQueryContext value)
      (error "invalid Org Element query context"))
    value))

(def (context-graph context)
  (unless (org-element-query-context? context)
    (error "Org Element query requires an admitted context" context))
  (.ref context 'graph))

(def (org-element-property context record name)
  ((org-element-graph-field-of (context-graph context)) record name))

(def (org-element-map context kind predicate)
  (context-graph context)
  (graph-query-map (.ref context 'index) kind predicate))

(def (org-element-lineage? context ancestor descendant)
  (context-graph context)
  (graph-query-lineage? (.ref context 'index) ancestor descendant))

(def (org-element-select query context scope-id targets)
  (unless (org-element-query? query)
    (error "Org Element select requires an admitted query" query))
  (context-graph context)
  (graph-query-select
   (.ref context 'index)
   (org-element-query-node-kind query)
   scope-id
   (org-element-query-relation query)
   targets
     (lambda (record)
       (ormap
        (lambda (group)
          (andmap
           (lambda (clause)
             (let* ((actual (org-element-property
                             context record
                             (org-element-clause-name clause)))
                    (expected (org-element-clause-value clause))
                    (matcher (org-element-clause-match clause))
                    (matches?
                     (lambda (value)
                       (and (string? value)
                            (case matcher
                              ((exact) (equal? value expected))
                              ((contains) (string-contains value expected)))))))
               (if (list? actual)
                 (ormap matches? actual)
                 (matches? actual))))
           group))
        (org-element-query-groups query)))))
