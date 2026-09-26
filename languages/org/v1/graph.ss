;;; -*- Gerbil -*-
;;; Parser projection of the Org-owned POO graph shape.

(import (only-in :gerbil-parser/graph-projection-support
                 make-graph-projection make-graph-node make-graph-field)
        (only-in "graph-shape.ss" org-v1-graph-shape
                 org-graph-node-rust org-graph-node-category
                 org-graph-node-label org-graph-node-fields
                 org-graph-field-rust org-graph-field-label
                 org-graph-field-mode))
(export org-v1-graph-projection)

(def (project-field value)
  (if (eq? (org-graph-field-mode value) 'one)
    (make-graph-field (org-graph-field-rust value)
                      (org-graph-field-label value))
    (make-graph-field (org-graph-field-rust value)
                      (org-graph-field-label value)
                      (org-graph-field-mode value))))

(def (project-node value)
  (make-graph-node (org-graph-node-rust value)
                   (org-graph-node-category value)
                   (org-graph-node-label value)
                   (map project-field (org-graph-node-fields value))))

(def org-v1-graph-projection
  (make-graph-projection (map project-node org-v1-graph-shape)))
