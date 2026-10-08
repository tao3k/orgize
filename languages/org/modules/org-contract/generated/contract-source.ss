;;; -*- Gerbil -*-
;;; @generated from the Org Element CST; do not edit.
(import (only-in "../interface.ss"
                 org-contract-block assert-org-element
                 make-org-contract-definition)
        (only-in "../../org-elements/interface.ss"
                 org-elements property property-contains all-of any-of at child-of descendant-of))
(export org-contract-definitions)
(def org-contract-definitions (list
  (make-org-contract-definition "section.scope.v1" 'subtree (org-contract-block
(assert-org-element "section.has-evidence-link" error
  (bindings
   (bind evidence
     (org-elements headline (property source-title "Evidence")
                   (child-of scope))))
  (org-elements link (descendant-of evidence))
  (expect at-least 1))
  ))
  (make-org-contract-definition "document.headlines.v1" 'document (org-contract-block
(assert-org-element "document.has-headline" warning
  (bindings)
  (org-elements headline (child-of scope))
  (expect at-least 2))
  ))
  (make-org-contract-definition "document.properties.v1" 'document (org-contract-block
(assert-org-element "document.has-contract-org-node-property" error
  (bindings)
  (org-elements node-property (property key "CONTRACT_ORG"))
  (expect at-least 1))
  ))
  (make-org-contract-definition "section.override-title.v1" 'subtree (org-contract-block
(assert-org-element "section.title-has-override" error
  (bindings)
  (org-elements headline (property-contains source-title "Override")
                (at scope))
  (expect at-least 1))
  ))
  (make-org-contract-definition "orgize.builtin.document-metadata.v1" 'document (org-contract-block
(assert-org-element "document.has-title-keyword" warning
  (bindings)
  (org-elements keyword (property key "TITLE"))
  (expect at-least 1)
  (message "document is missing a #+TITLE keyword")
  (fix "add a non-empty #+TITLE keyword near the top of the document"))
(assert-org-element "document.has-properties" warning
  (bindings)
  (org-elements keyword (property key "PROPERTY"))
  (expect at-least 1)
  (message "document is missing document-level properties")
  (fix "add document-level properties with #+PROPERTY metadata"))
  ))
))
