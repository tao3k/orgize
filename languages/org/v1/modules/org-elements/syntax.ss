;;; -*- Gerbil -*-
;;; Hygienic declarative Org Element query syntax over POO clauses.

(import (only-in "objects.ss"
                 make-org-element-property-clause
                 make-org-element-relation-clause)
        (only-in "funs.ss" org-element-query-compose))
(export org-elements property child-of descendant-of)

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
