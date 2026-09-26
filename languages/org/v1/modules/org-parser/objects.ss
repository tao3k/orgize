;;; -*- Gerbil -*-
;;; POO strategy values are projected to event IR before runtime.

(import (only-in :clan/poo/object .o .ref)
        (only-in "types.ss"
                 +org-event-block-kind+ +org-inline-markup-kind+
                 org-event-block? org-inline-markup?))
(export make-org-event-block org-event-block-id org-event-block-rule
        make-org-inline-markup org-inline-markup-byte
        org-inline-markup-id org-inline-markup-node)

(def (make-org-event-block id-value rule-value)
  (let (value (.o kind: +org-event-block-kind+
                  id: id-value rule: rule-value))
    (unless (org-event-block? value)
      (error "invalid Org event block strategy" value))
    value))

(def (org-event-block-id value) (.ref value 'id))
(def (org-event-block-rule value) (.ref value 'rule))

(def (make-org-inline-markup byte-value id-value node-value)
  (let (value (.o kind: +org-inline-markup-kind+
                  byte: byte-value id: id-value node: node-value))
    (unless (org-inline-markup? value)
      (error "invalid Org inline markup strategy" value))
    value))

(def (org-inline-markup-byte value) (.ref value 'byte))
(def (org-inline-markup-id value) (.ref value 'id))
(def (org-inline-markup-node value) (.ref value 'node))
