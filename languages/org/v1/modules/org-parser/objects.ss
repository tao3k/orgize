;;; -*- Gerbil -*-
;;; POO strategy values are projected to event IR before runtime.

(import (only-in :clan/poo/object .o .ref)
        (only-in "types.ss"
                 +org-event-block-kind+ +org-named-block-kind+
                 +org-inline-markup-kind+
                 +org-event-strategy-kind+
                 org-event-block? org-named-block?
                 org-inline-markup? org-event-strategy?))
(export make-org-event-block org-event-block-id org-event-block-rule
        make-org-named-block org-named-block-opening
        org-named-block-closing org-named-block-node
        org-named-block-name-token
        make-org-inline-markup org-inline-markup-byte
        org-inline-markup-id org-inline-markup-node
        make-org-event-strategy org-event-strategy-root
        org-event-strategy-initial org-event-strategy-line-forms
        org-event-strategy-finish-forms org-event-strategy-helpers)

(def (make-org-event-block id-value rule-value)
  (let (value (.o kind: +org-event-block-kind+
                  id: id-value rule: rule-value))
    (unless (org-event-block? value)
      (error "invalid Org event block strategy" value))
    value))

(def (org-event-block-id value) (.ref value 'id))
(def (org-event-block-rule value) (.ref value 'rule))

(def (make-org-named-block opening-value closing-value node-value name-token-value)
  (let (value (.o kind: +org-named-block-kind+
                  opening: opening-value closing: closing-value
                  node: node-value name-token: name-token-value))
    (unless (org-named-block? value)
      (error "invalid Org named block strategy" value))
    value))

(def (org-named-block-opening value) (.ref value 'opening))
(def (org-named-block-closing value) (.ref value 'closing))
(def (org-named-block-node value) (.ref value 'node))
(def (org-named-block-name-token value) (.ref value 'name-token))

(def (make-org-inline-markup byte-value id-value node-value)
  (let (value (.o kind: +org-inline-markup-kind+
                  byte: byte-value id: id-value node: node-value))
    (unless (org-inline-markup? value)
      (error "invalid Org inline markup strategy" value))
    value))

(def (org-inline-markup-byte value) (.ref value 'byte))
(def (org-inline-markup-id value) (.ref value 'id))
(def (org-inline-markup-node value) (.ref value 'node))

(def (make-org-event-strategy root-value initial-value line-forms-value
                              finish-forms-value helpers-value)
  (let (value (.o kind: +org-event-strategy-kind+
                  root: root-value initial: initial-value
                  line-forms: line-forms-value
                  finish-forms: finish-forms-value
                  helpers: helpers-value))
    (unless (org-event-strategy? value)
      (error "invalid Org event strategy" value))
    value))

(def (org-event-strategy-root value) (.ref value 'root))
(def (org-event-strategy-initial value) (.ref value 'initial))
(def (org-event-strategy-line-forms value) (.ref value 'line-forms))
(def (org-event-strategy-finish-forms value) (.ref value 'finish-forms))
(def (org-event-strategy-helpers value) (.ref value 'helpers))
