;;; -*- Gerbil -*-
;;; Admitted POO declarations for Org Element graph projection.

(import (only-in :clan/poo/object .ref .slot? object?)
        (only-in :clan/poo/mop define-type Type. element?)
        (only-in :std/list/list every))
(export +org-graph-node-kind+ +org-graph-field-kind+
        OrgGraphNode OrgGraphField org-graph-node? org-graph-field?)

(def +org-graph-node-kind+ 'org-graph-node)
(def +org-graph-field-kind+ 'org-graph-field)

(def (org-graph-field-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'rust)
       (.slot? value 'label) (.slot? value 'mode)
       (eq? (.ref value 'kind) +org-graph-field-kind+)
       (symbol? (.ref value 'rust))
       (string? (.ref value 'label))
       (and (memq (.ref value 'mode) '(one each append-or-empty)) #t)))

(define-type (OrgGraphField @ Type.)
  .element?: org-graph-field-shape?)

(def (org-graph-node-shape? value)
  (and (object? value)
       (.slot? value 'kind) (.slot? value 'rust)
       (.slot? value 'category) (.slot? value 'label)
       (.slot? value 'fields)
       (eq? (.ref value 'kind) +org-graph-node-kind+)
       (symbol? (.ref value 'rust))
       (member (.ref value 'category) '("document" "section" "element"
                                         "object" "property"))
       (string? (.ref value 'label))
       (list? (.ref value 'fields))
       (every org-graph-field? (.ref value 'fields))))

(define-type (OrgGraphNode @ Type.)
  .element?: org-graph-node-shape?)

(def (org-graph-field? value) (element? OrgGraphField value))
(def (org-graph-node? value) (element? OrgGraphNode value))
