;;; -*- Gerbil -*-
;;; @generated from the Org Element CST; do not edit.
(import (only-in "../../../../languages/org/modules/org-contract/interface.ss"
                 org-contract-block assert-org-element
                 make-org-contract-definition)
        (only-in "../../../../languages/org/modules/org-elements/interface.ss"
                 org-elements property property-contains all-of any-of at child-of descendant-of))
(export org-contract-definitions)
(def org-contract-definitions (list
  (make-org-contract-definition "customer.review-evidence" 'subtree (org-contract-block
(assert-org-element "customer.review-has-link" error
  (bindings)
  (org-elements link (descendant-of scope))
  (expect at-least 1)
  (message "review requires a link")
  (fix "add a linked source to the review"))
  ))
  (make-org-contract-definition "customer.has-task" 'document (org-contract-block
(assert-org-element "customer.has-task-headline" warning
  (bindings)
  (org-elements headline
    (all-of (property todo-type "todo")
            (any-of (property-contains source-title "Review")
                    (property-contains source-title "Audit"))))
  (expect at-least 1))
  ))
))
