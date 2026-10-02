;;; -*- Gerbil -*-
;;; Source-owned graph declarations projected to Rust/Rowan at build time.

(import (only-in :clan/poo/object .o .ref)
        (only-in "graph-types.ss"
                 +org-graph-node-kind+ +org-graph-field-kind+
                 org-graph-node? org-graph-field?))
(export make-org-graph-node make-org-graph-field
        org-graph-node-rust org-graph-node-category
        org-graph-node-label org-graph-node-fields
        org-graph-field-rust org-graph-field-label org-graph-field-mode)

(def (make-org-graph-node rust-value category-value label-value fields-value)
  (let (value (.o kind: +org-graph-node-kind+
                  rust: rust-value category: category-value
                  label: label-value fields: fields-value))
    (unless (org-graph-node? value)
      (error "invalid Org graph node declaration" value))
    value))

(def (make-org-graph-field rust-value label-value mode-value)
  (let (value (.o kind: +org-graph-field-kind+
                  rust: rust-value label: label-value mode: mode-value))
    (unless (org-graph-field? value)
      (error "invalid Org graph field declaration" value))
    value))

(def (org-graph-node-rust value) (.ref value 'rust))
(def (org-graph-node-category value) (.ref value 'category))
(def (org-graph-node-label value) (.ref value 'label))
(def (org-graph-node-fields value) (.ref value 'fields))
(def (org-graph-field-rust value) (.ref value 'rust))
(def (org-graph-field-label value) (.ref value 'label))
(def (org-graph-field-mode value) (.ref value 'mode))
