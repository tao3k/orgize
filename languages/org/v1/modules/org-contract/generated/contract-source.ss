;;; -*- Gerbil -*-
;;; @generated from the Org Element CST; do not edit.
(import (only-in "../interface.ss"
                 org-contract-block assert-org-element
                 make-org-contract-definition)
        (only-in "../../org-elements/interface.ss"
                 org-elements property child-of descendant-of))
(export org-contract-definition)
(def org-contract-definition
  (make-org-contract-definition "section.scope.v1" 'subtree (org-contract-block
(assert-org-element "section.has-evidence-link" error
  (bindings
   (bind evidence
     (org-elements headline (property title "Evidence")
                   (child-of scope))))
  (org-elements link (descendant-of evidence))
  (expect at-least 1))
  )))
