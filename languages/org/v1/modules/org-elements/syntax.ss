;;; -*- Gerbil -*-
;;; Hygienic declarative Org Element query syntax over POO clauses.

(import (only-in "objects.ss"
                 make-org-element-property-clause
                 make-org-element-relation-clause
                 make-org-named-element-query)
        (only-in "funs.ss" org-element-query-compose
                 org-element-predicate-all org-element-predicate-any))
(export org-elements org-element-query
        property property-contains all-of any-of
        at child-of descendant-of)

;; A tagged :org-elements block admits one named query declaration only.
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

(defsyntax (any-of stx)
  (syntax-case stx ()
    ((_ clause clauses ...)
     (syntax (org-element-predicate-any (list clause clauses ...))))))

(defsyntax (at stx)
  (syntax-case stx (scope)
    ((_ scope)
     (syntax (make-org-element-relation-clause 'at 'scope)))
    ((_ binding-name)
     (syntax (make-org-element-relation-clause
              'at (symbol->string 'binding-name))))))

(defsyntax (child-of stx)
  (syntax-case stx (scope)
    ((_ scope)
     (syntax (make-org-element-relation-clause 'child-of 'scope)))
    ((_ binding-name)
     (syntax (make-org-element-relation-clause
              'child-of (symbol->string 'binding-name))))))

(defsyntax (descendant-of stx)
  (syntax-case stx (scope)
    ((_ scope)
     (syntax (make-org-element-relation-clause 'descendant-of 'scope)))
    ((_ binding-name)
     (syntax (make-org-element-relation-clause
              'descendant-of (symbol->string 'binding-name))))))
