;;; -*- Gerbil -*-
;;; Hygienic, bounded declarations inside Scheme :org-contract blocks.

(import (only-in "../org-elements/runtime-interface.ss" org-elements)
        (only-in "objects.ss"
                 make-org-contract-expectation
                 make-org-contract-binding make-org-contract-assertion))
(export org-contract-block assert-org-element)

;;; The generated module supplies this wrapper; authors write only assertions.
;;; Reject arbitrary Scheme forms before any block expression is evaluated.
(defsyntax (org-contract-block stx)
  (syntax-case stx (assert-org-element)
    ((_ (assert-org-element id severity bindings query expectation) ...)
     (syntax (list (assert-org-element id severity bindings
                                        query expectation) ...)))))

(defsyntax (org-contract-severity stx)
  (syntax-case stx (error warning info)
    ((_ error) (syntax 'error))
    ((_ warning) (syntax 'warning))
    ((_ info) (syntax 'info))))

(defsyntax (org-contract-expect stx)
  (syntax-case stx (at-least exactly at-most)
    ((_ at-least count)
     (syntax (make-org-contract-expectation 'at-least count)))
    ((_ exactly count)
     (syntax (make-org-contract-expectation 'exactly count)))
    ((_ at-most count)
     (syntax (make-org-contract-expectation 'at-most count)))))

(defsyntax (assert-org-element stx)
  (syntax-case stx (bindings bind org-elements expect)
    ((_ id severity
        (bindings
         (bind binding-name
           (org-elements binding-kind binding-clause ...)) ...)
        (org-elements kind query-clause ...)
        (expect operator count))
     (syntax
      (make-org-contract-assertion
       id (org-contract-severity severity)
       (org-elements kind query-clause ...)
       (org-contract-expect operator count)
       (list (make-org-contract-binding
              (symbol->string 'binding-name)
              (org-elements binding-kind binding-clause ...)) ...))))))
