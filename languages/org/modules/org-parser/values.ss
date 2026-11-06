;;; -*- Gerbil -*-
;;; POO strategy values are projected to event IR before runtime.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop validate)
        (only-in :yuanxu/object-family/interface defpoo-object-family)
        (only-in :gerbil-parser/event-fold-program-support prepare-event-fold-program)
        (only-in "value-types.ss"
                 +org-event-block-kind+ +org-named-block-kind+
                 +org-inline-markup-kind+
                 +org-inline-script-kind+
                 +org-event-helper-kind+
                 +org-event-strategy-kind+
                 OrgEventBlock OrgNamedBlock OrgInlineMarkup OrgInlineScript
                 OrgEventHelper OrgEventStrategy))
(export make-org-event-block org-event-block-id org-event-block-rule
        make-org-named-block org-named-block-opening
        org-named-block-closing org-named-block-node
        org-named-block-name-token
        make-org-inline-markup org-inline-markup-byte
        org-inline-markup-id org-inline-markup-node
        make-org-inline-script org-inline-script-byte
        org-inline-script-id org-inline-script-node
        make-org-event-helper org-event-helper-name
        org-event-helper-initial org-event-helper-forms
        org-event-helper-parameters
        org-event-helper-descriptor
        make-org-event-strategy org-event-strategy-root
        org-event-strategy-initial org-event-strategy-line-forms
        org-event-strategy-finish-forms org-event-strategy-helpers
        org-event-strategy-program org-event-strategy-helper-descriptors
        org-event-strategy-parameters)

(def (make-org-event-block id-value rule-value)
  (let (value (.o kind: +org-event-block-kind+
                  id: id-value rule: rule-value))
    (validate OrgEventBlock value)))

(def (make-org-named-block opening-value closing-value node-value name-token-value)
  (let (value (.o kind: +org-named-block-kind+
                  opening: opening-value closing: closing-value
                  node: node-value name-token: name-token-value))
    (validate OrgNamedBlock value)))

(def (make-org-inline-markup byte-value id-value node-value)
  (let (value (.o kind: +org-inline-markup-kind+
                  byte: byte-value id: id-value node: node-value))
    (validate OrgInlineMarkup value)))

(def (make-org-inline-script byte-value id-value node-value)
  (let (value (.o kind: +org-inline-script-kind+
                  byte: byte-value id: id-value node: node-value))
    (validate OrgInlineScript value)))

(def (make-org-event-helper name-value initial-value forms-value
                            (parameters-value '()))
  (let (value (.o kind: +org-event-helper-kind+
                  name: name-value initial: initial-value forms: forms-value
                  parameters: parameters-value))
    (validate OrgEventHelper value)))

(def (org-event-helper-descriptor value)
  (let (descriptor
        (list (org-event-helper-name value)
              (org-event-helper-initial value)
              (org-event-helper-forms value)))
    (if (null? (org-event-helper-parameters value)) descriptor
      (append descriptor (list (org-event-helper-parameters value))))))

(def (make-org-event-strategy root-value initial-value line-forms-value
                              finish-forms-value helpers-value
                              (parameters-value '()))
  (let (value (.o (:: self) kind: +org-event-strategy-kind+
                  root: root-value initial: initial-value
                  line-forms: line-forms-value
                  finish-forms: finish-forms-value
                  helpers: helpers-value
                  helper-descriptors: (map org-event-helper-descriptor (.ref self 'helpers))
                  program: (prepare-event-fold-program
                            (.ref self 'root) (.ref self 'initial)
                            (.ref self 'line-forms) (.ref self 'finish-forms)
                            (.ref self 'helper-descriptors))
                  parameters: parameters-value))
    (validate OrgEventStrategy value)))

;; The immutable strategy owns its derived IR; document calls only read it.

(defpoo-object-family
  (accessors
   (org-event-block-id id)
   (org-event-block-rule rule)
   (org-named-block-opening opening)
   (org-named-block-closing closing)
   (org-named-block-node node)
   (org-named-block-name-token name-token)
   (org-inline-markup-byte byte)
   (org-inline-markup-id id)
   (org-inline-markup-node node)
   (org-inline-script-byte byte)
   (org-inline-script-id id)
   (org-inline-script-node node)
   (org-event-helper-name name)
   (org-event-helper-initial initial)
   (org-event-helper-forms forms)
   (org-event-helper-parameters parameters)
   (org-event-strategy-root root)
   (org-event-strategy-initial initial)
   (org-event-strategy-line-forms line-forms)
   (org-event-strategy-finish-forms finish-forms)
   (org-event-strategy-helpers helpers)
   (org-event-strategy-parameters parameters)
   (org-event-strategy-program program)
   (org-event-strategy-helper-descriptors helper-descriptors)
)
  (projections))
