;;; -*- Gerbil -*-
;;; Pure Org Element map, property, lineage, and scoped selection.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop element?)
        (only-in "types.ss"
                 +org-element-schema+ +org-element-context-kind+
                 OrgElementQueryContext org-element-query?
                 org-element-query-clause?
                 org-element-graph-view? org-element-query-context?)
        (only-in "objects.ss"
                 make-org-element-query org-element-clause-kind
                 org-element-clause-name org-element-clause-value
                 org-element-clause-match
                 org-element-query-node-kind org-element-query-field-name
                 org-element-query-field-value org-element-query-field-match
                 org-element-query-relation
                 org-element-graph-records org-element-graph-id-of
                 org-element-graph-parent-of org-element-graph-kind-of
                 org-element-graph-field-of))
(export org-element-query-compose make-org-element-query-context org-element-map
        org-element-property org-element-lineage? org-element-select)

(defstruct element-index (by-id count))

(def (org-element-query-compose kind clauses)
  (let loop ((rest clauses) (field-name #f) (field-value #f)
             (field-match 'exact)
             (relation 'any) (target #f))
    (if (null? rest)
      (make-org-element-query kind field-name field-value
                              relation target field-match)
      (let (clause (car rest))
        (unless (org-element-query-clause? clause)
          (error "Org Element query requires POO clauses" clause))
        (case (org-element-clause-kind clause)
          ((property)
           (when field-name
             (error "duplicate Org Element property clause" kind))
           (loop (cdr rest) (org-element-clause-name clause)
                 (org-element-clause-value clause)
                 (org-element-clause-match clause) relation target))
          ((relation)
           (unless (eq? relation 'any)
             (error "duplicate Org Element relation clause" kind))
           (loop (cdr rest) field-name field-value field-match
                 (org-element-clause-name clause)
                 (org-element-clause-value clause))))))))

(def (make-org-element-query-context graph-value)
  (unless (org-element-graph-view? graph-value)
    (error "Org Element query requires an admitted graph view" graph-value))
  (let (by-id (make-hash-table))
    (for-each
     (lambda (record)
       (let (id ((org-element-graph-id-of graph-value) record))
         (when (hash-get by-id id)
           (error "duplicate Org Element id" id))
         (hash-put! by-id id record)))
     (org-element-graph-records graph-value))
    (let (value
          (.o kind: +org-element-context-kind+
              schema: +org-element-schema+
              graph: graph-value
              index: (make-element-index
                      by-id (length (org-element-graph-records graph-value)))))
      (unless (element? OrgElementQueryContext value)
        (error "invalid Org Element query context"))
      value)))

(def (context-graph context)
  (unless (org-element-query-context? context)
    (error "Org Element query requires an admitted context" context))
  (.ref context 'graph))

(def (org-element-property context record name)
  ((org-element-graph-field-of (context-graph context)) record name))

(def (org-element-map context kind predicate)
  (let (graph (context-graph context))
    (filter
     (lambda (record)
       (and (equal? ((org-element-graph-kind-of graph) record) kind)
            (predicate record)))
     (org-element-graph-records graph))))

(def (org-element-lineage? context ancestor descendant)
  (let* ((graph (context-graph context))
         (index (.ref context 'index))
         (by-id (element-index-by-id index))
         (parent-of (org-element-graph-parent-of graph)))
    (let loop ((id descendant) (remaining (element-index-count index)))
      (and (> remaining 0)
           (let (record (hash-get by-id id))
             (and record
                  (let (parent (parent-of record))
                    (and parent
                         (or (equal? parent ancestor)
                             (loop parent (- remaining 1)))))))))))

(def (org-element-select query context scope-id targets)
  (unless (org-element-query? query)
    (error "Org Element select requires an admitted query" query))
  (let* ((graph (context-graph context))
         (id-of (org-element-graph-id-of graph))
         (parent-of (org-element-graph-parent-of graph))
         (field-name (org-element-query-field-name query))
         (field-value (org-element-query-field-value query))
         (field-match (org-element-query-field-match query))
         (relation (org-element-query-relation query)))
    (org-element-map
     context (org-element-query-node-kind query)
     (lambda (record)
       (let (id (id-of record))
         (and (or (equal? id scope-id)
                  (org-element-lineage? context scope-id id))
              (or (not field-name)
                  (let (actual (org-element-property context record
                                                      field-name))
                    (let (matches?
                          (lambda (value)
                            (and (string? value)
                                 (case field-match
                                   ((exact) (equal? value field-value))
                                   ((contains)
                                    (if (string-contains value field-value)
                                      #t #f))))))
                      (if (list? actual)
                        (ormap matches? actual)
                        (matches? actual)))))
              (case relation
                ((any) #t)
                ((at) (member id targets))
                ((child-of) (member (parent-of record) targets))
                ((descendant-of)
                 (ormap (lambda (target)
                          (org-element-lineage? context target id))
                        targets))
                (else (error "unknown Org Element relation" relation)))))))))
