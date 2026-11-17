;;; -*- Gerbil -*-
;;; Hygienic declarative Org Element query syntax over POO clauses.

(import (only-in "values.ss"
                 make-org-element-property-clause
                 make-org-element-relation-clause
                 make-org-named-element-query)
        (only-in "query.ss" org-element-query-compose
                 org-element-predicate-all org-element-predicate-any))
(export org-elements org-element-query
        property property-contains property-local-path all-of any-of
        at child-of descendant-of)

;; A tagged :org-elements-query block admits one named query declaration only.
(defsyntax (org-element-query stx)
  (syntax-case stx (org-elements)
    ((_ id (org-elements kind clause ...))
     (syntax (make-org-named-element-query
              id (org-elements kind clause ...))))))

(defsyntax (org-elements stx)
  (syntax-case stx ()
    ((_ kind clause ...)
     (syntax (org-element-query-compose (symbol->string 'kind)
                                        (list clause ...))))))

(defsyntax (property stx)
  (syntax-case stx ()
    ((_ name value)
     (syntax (make-org-element-property-clause
              (symbol->string 'name) value)))))

(defsyntax (property-contains stx)
  (syntax-case stx ()
    ((_ name value)
     (syntax (make-org-element-property-clause
              (symbol->string 'name) value 'contains)))))

(defsyntax (all-of stx)
  (syntax-case stx ()
    ((_ clause clauses ...)
     (syntax (org-element-predicate-all (list clause clauses ...))))))

;; A native field-format predicate, not a blacklist of host directory names.
(defsyntax (property-local-path stx)
  (syntax-case stx ()
    ((_ name)
     (syntax (make-org-element-property-clause
              (symbol->string 'name) "" 'local-path)))))

(defsyntax (any-of stx)
  (syntax-case stx ()
    ((_ clause clauses ...)
     (syntax (org-element-predicate-any (list clause clauses ...))))))

;; Relation targets name Org query bindings. The reserved scope target does
;; not depend on a Scheme binding with the same spelling at the call site.
(defsyntax (org-element-relation stx)
  (syntax-case stx ()
    ((_ relation target)
     (identifier? (syntax target))
     (if (eq? (stx-e (syntax target)) 'scope)
         (syntax (make-org-element-relation-clause 'relation 'scope))
         (syntax (make-org-element-relation-clause 'relation (symbol->string 'target)))))))

(defrule (at target) (org-element-relation at target))
(defrule (child-of target) (org-element-relation child-of target))
(defrule (descendant-of target) (org-element-relation descendant-of target))
