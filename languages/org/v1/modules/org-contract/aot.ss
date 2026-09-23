;;; -*- Gerbil -*-
;;; Private AOT projection of admitted Scheme/POO contracts to Rust values.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 rust-struct rust-static rust-array rust-some rust-none
                 rust-number rust-string rust-identifier rust-render)
        (only-in :gerbil-parser/graph-projection-support
                 graph-projection-digest)
        (only-in "../../grammar.ss" org-v1-language-grammar)
        (only-in "../../graph.ss" org-v1-graph-projection)
        (only-in "../org-elements/interface.ss"
                 org-element-query-node-kind org-element-query-field-name
                 org-element-query-field-value org-element-query-field-match
                 org-element-query-relation
                 org-element-query-target)
        (only-in "types.ss" org-contract-definition?)
        (only-in "objects.ss"
                 org-contract-expectation-operator org-contract-expectation-count
                 org-contract-binding-name org-contract-binding-query
                 org-contract-assertion-id org-contract-assertion-severity
                 org-contract-assertion-bindings org-contract-assertion-query
                 org-contract-assertion-expectation
                 org-contract-definition-id org-contract-definition-scope
                 org-contract-definition-assertions))
(export org-contract-rust-syntax org-contract-rust-source
        generate-org-contract-rust-module)

(def (optional-string value)
  (if value (rust-some (rust-string value)) (rust-none)))

(def (relation-value relation)
  (rust-identifier
   (case relation
     ((any) "ContractRelation::Any")
     ((at) "ContractRelation::At")
     ((child-of) "ContractRelation::ChildOf")
     ((descendant-of) "ContractRelation::DescendantOf")
     (else (error "invalid Org contract relation" relation)))))

(def (operator-value operator)
  (rust-identifier
   (case operator
     ((at-least) "ContractOperator::AtLeast")
     ((exactly) "ContractOperator::Exactly")
     ((at-most) "ContractOperator::AtMost")
     (else (error "invalid Org contract operator" operator)))))

(def (severity-value severity)
  (rust-identifier
   (case severity
     ((error) "ContractSeverity::Error")
     ((warning) "ContractSeverity::Warning")
     ((info) "ContractSeverity::Info")
     (else (error "invalid Org contract severity" severity)))))

(def (scope-value scope)
  (rust-identifier
   (case scope
     ((document) "ContractScope::Document")
     ((subtree) "ContractScope::Subtree")
     (else (error "invalid Org contract scope" scope)))))

(def (query-value query)
  (let (target (org-element-query-target query))
    (rust-struct ContractQueryRule
      (node_kind (rust-string (org-element-query-node-kind query)))
      (field_name (optional-string (org-element-query-field-name query)))
      (field_value (optional-string (org-element-query-field-value query)))
      (field_match
       (rust-identifier
        (case (org-element-query-field-match query)
          ((exact) "ContractFieldMatch::Exact")
          ((contains) "ContractFieldMatch::Contains")
          (else (error "invalid Org Element field match")))))
      (relation (relation-value (org-element-query-relation query)))
      (target_scope (rust-identifier (if (eq? target 'scope) "true" "false")))
      (target_binding (optional-string (and (string? target) target))))))

(def (expectation-value expectation)
  (rust-struct ContractExpectationRule
    (operator (operator-value (org-contract-expectation-operator expectation)))
    (count (rust-number (org-contract-expectation-count expectation)))))

(def (binding-value binding)
  (rust-struct ContractBindingRule
    (name (rust-string (org-contract-binding-name binding)))
    (query (query-value (org-contract-binding-query binding)))))

(def (assertion-value assertion)
  (rust-struct ContractAssertionRule
    (id (rust-string (org-contract-assertion-id assertion)))
    (severity (severity-value (org-contract-assertion-severity assertion)))
    (bindings (rust-array (map binding-value
                               (org-contract-assertion-bindings assertion))))
    (query (query-value (org-contract-assertion-query assertion)))
    (expectation (expectation-value
                  (org-contract-assertion-expectation assertion)))))

(def (contract-value definition)
  (unless (org-contract-definition? definition)
    (error "Org Contract AOT requires an admitted POO definition"))
  (rust-struct ContractRule
    (id (rust-string (org-contract-definition-id definition)))
    (graph_digest
     (rust-string
      (graph-projection-digest org-v1-language-grammar
                               org-v1-graph-projection)))
    (scope (scope-value (org-contract-definition-scope definition)))
    (assertions (rust-array
                 (map assertion-value
                      (org-contract-definition-assertions definition))))))

(def (org-contract-rust-syntax definitions)
  (unless (and (pair? definitions)
               (let loop ((rest definitions) (seen '()))
                 (or (null? rest)
                     (let (definition (car rest))
                       (and (org-contract-definition? definition)
                            (not (member (org-contract-definition-id definition)
                                         seen))
                            (loop (cdr rest)
                                  (cons (org-contract-definition-id definition)
                                        seen)))))))
    (error "Org Contract AOT requires distinct admitted POO definitions"))
  (rust-static CONTRACTS ContractPack
    (rust-struct ContractPack
      (rules (rust-array (map contract-value definitions))))))

(def (org-contract-rust-source definitions)
  (rust-render (org-contract-rust-syntax definitions)))

(def (generate-org-contract-rust-module output-path definitions)
  (let (source (org-contract-rust-source definitions))
    (call-with-output-file output-path
      (lambda (port) (display source port)))))
