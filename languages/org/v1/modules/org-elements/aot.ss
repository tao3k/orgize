;;; -*- Gerbil -*-
;;; Org Element POO queries -> typed Rust AOT pack, independent of Contract.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 rust-struct rust-static rust-array rust-some rust-none
                 rust-string rust-identifier rust-render)
        (only-in :gerbil-parser/graph-projection-support
                 graph-projection-digest)
        (only-in "../../grammar.ss" org-v1-language-grammar)
        (only-in "../../graph.ss" org-v1-graph-projection)
        (only-in "types.ss" org-named-element-query?)
        (only-in "objects.ss"
                 org-named-element-query-id org-named-element-query-query
                 org-element-query-node-kind org-element-query-field-name
                 org-element-query-field-value org-element-query-field-match
                 org-element-query-relation org-element-query-target))
(export org-element-query-rust-syntax org-element-query-rust-source
        generate-org-element-query-rust-module)

(def (optional-string value)
  (if value (rust-some (rust-string value)) (rust-none)))

(def (field-match-value value)
  (rust-identifier
   (case value
     ((exact) "OrgElementFieldMatch::Exact")
     ((contains) "OrgElementFieldMatch::Contains")
     (else (error "unsupported Org Element match" value)))))

(def (relation-value value)
  (rust-identifier
   (case value
     ((any) "OrgElementRelation::Any")
     ((at) "OrgElementRelation::At")
     ((child-of) "OrgElementRelation::ChildOf")
     ((descendant-of) "OrgElementRelation::DescendantOf")
     (else (error "unsupported Org Element relation" value)))))

(def (query-value named)
  (unless (org-named-element-query? named)
    (error "query AOT requires an admitted POO value" named))
  (let* ((query (org-named-element-query-query named))
         (target (org-element-query-target query))
         (field (org-element-query-field-name query)))
    (unless (or (not target) (eq? target 'scope))
      (error "named Element queries cannot use unbound targets" target))
    (when (member field '("title" "raw-value" "todo-keyword"
                          "priority" "tags"))
      (error "derived Element property lacks an AOT implementation" field))
    (rust-struct OrgElementQueryRule
      (id (rust-string (org-named-element-query-id named)))
      (node_kind (rust-string (org-element-query-node-kind query)))
      (field_name (optional-string (org-element-query-field-name query)))
      (field_value (optional-string (org-element-query-field-value query)))
      (field_match (field-match-value (org-element-query-field-match query)))
      (relation (relation-value (org-element-query-relation query)))
      (target_scope (rust-identifier (if (eq? target 'scope) "true" "false"))))))

(def (distinct-queries? queries)
  (let loop ((remaining queries) (seen '()))
    (or (null? remaining)
        (let (query (car remaining))
          (and (org-named-element-query? query)
               (not (member (org-named-element-query-id query) seen))
               (loop (cdr remaining)
                     (cons (org-named-element-query-id query) seen)))))))

(def (org-element-query-rust-syntax queries)
  (unless (and (pair? queries) (distinct-queries? queries))
    (error "query AOT requires distinct admitted named queries" queries))
  (rust-static QUERIES OrgElementQueryPack
    (rust-struct OrgElementQueryPack
      (graph_digest
       (rust-string (graph-projection-digest
                     org-v1-language-grammar org-v1-graph-projection)))
      (rules (rust-array (map query-value queries))))))

(def (org-element-query-rust-source queries)
  (rust-render (org-element-query-rust-syntax queries)))

(def (generate-org-element-query-rust-module output-path queries)
  (call-with-output-file output-path
    (lambda (port)
      (write-string (org-element-query-rust-source queries) port))))
