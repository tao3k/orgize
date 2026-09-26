;;; -*- Gerbil -*-
;;; Org contract feature boundaries.  This is not an expression grammar.

(import (only-in :clan/poo/object .ref .slot? object?)
        (only-in :clan/poo/mop define-type Type. element?)
        (only-in :std/list/list every)
        (only-in "../org-elements/types.ss" org-element-query?))
(export +org-contract-schema+
        +org-contract-expectation-kind+
        +org-contract-binding-kind+ +org-contract-assertion-kind+
        +org-contract-definition-kind+ +org-contract-result-kind+
        +org-contract-profile-kind+
        OrgContractExpectation OrgContractBinding
        OrgContractAssertion OrgContractDefinition OrgContractResult
        OrgContractProfile org-contract-expectation?
        org-contract-binding? org-contract-assertion?
        org-contract-definition? org-contract-result?
        org-contract-profile?)

(def +org-contract-schema+ "orgize.org-contract.v1")
(def +org-contract-expectation-kind+ 'org-contract-expectation)
(def +org-contract-binding-kind+ 'org-contract-binding)
(def +org-contract-assertion-kind+ 'org-contract-assertion)
(def +org-contract-definition-kind+ 'org-contract-definition)
(def +org-contract-result-kind+ 'org-contract-result)
(def +org-contract-profile-kind+ 'org-contract-profile)

(def (has-kind-and-slots? value kind slots)
  (and (object? value)
       (.slot? value 'kind)
       (eq? (.ref value 'kind) kind)
       (every (lambda (slot) (.slot? value slot)) slots)))

(def (nonempty-string? value)
  (and (string? value) (> (string-length value) 0)))

(def (org-contract-expectation-shape? value)
  (and (has-kind-and-slots? value +org-contract-expectation-kind+
                            '(schema operator count))
       (equal? (.ref value 'schema) +org-contract-schema+)
       (memq (.ref value 'operator) '(at-least exactly at-most))
       (exact-integer? (.ref value 'count))
       (>= (.ref value 'count) 0)))

(define-type (OrgContractExpectation @ Type.)
  .element?: org-contract-expectation-shape?)

(def (org-contract-binding-shape? value)
  (and (has-kind-and-slots? value +org-contract-binding-kind+
                            '(schema name query))
       (equal? (.ref value 'schema) +org-contract-schema+)
       (nonempty-string? (.ref value 'name))
       (org-element-query? (.ref value 'query))))

(define-type (OrgContractBinding @ Type.)
  .element?: org-contract-binding-shape?)

(def (org-contract-assertion-shape? value)
  (and (has-kind-and-slots? value +org-contract-assertion-kind+
                            '(schema id severity bindings query expectation))
       (equal? (.ref value 'schema) +org-contract-schema+)
       (nonempty-string? (.ref value 'id))
       (memq (.ref value 'severity) '(error warning info))
       (list? (.ref value 'bindings))
       (every (lambda (binding) (element? OrgContractBinding binding))
              (.ref value 'bindings))
       (org-element-query? (.ref value 'query))
       (element? OrgContractExpectation (.ref value 'expectation))))

(define-type (OrgContractAssertion @ Type.)
  .element?: org-contract-assertion-shape?)

(def (org-contract-definition-shape? value)
  (and (has-kind-and-slots? value +org-contract-definition-kind+
                            '(schema id scope assertions))
       (equal? (.ref value 'schema) +org-contract-schema+)
       (nonempty-string? (.ref value 'id))
       (memq (.ref value 'scope) '(document subtree))
       (list? (.ref value 'assertions))
       (every (lambda (assertion)
                (element? OrgContractAssertion assertion))
              (.ref value 'assertions))))

(define-type (OrgContractDefinition @ Type.)
  .element?: org-contract-definition-shape?)

(def (org-contract-result-shape? value)
  (and (has-kind-and-slots? value +org-contract-result-kind+
                            '(schema assertion-id matched-count passed?))
       (equal? (.ref value 'schema) +org-contract-schema+)
       (nonempty-string? (.ref value 'assertion-id))
       (exact-integer? (.ref value 'matched-count))
       (>= (.ref value 'matched-count) 0)
       (boolean? (.ref value 'passed?))))

(define-type (OrgContractResult @ Type.)
  .element?: org-contract-result-shape?)

(def (org-contract-profile-shape? value)
  (and (has-kind-and-slots?
        value +org-contract-profile-kind+
        '(schema source-form aot-target runtime-owner))
       (equal? (.ref value 'schema) +org-contract-schema+)
       (eq? (.ref value 'source-form) 'org-headings-and-properties)
       (eq? (.ref value 'aot-target) 'rust)
       (equal? (.ref value 'runtime-owner) "orgize")))

(define-type (OrgContractProfile @ Type.)
  .element?: org-contract-profile-shape?)

(def (org-contract-expectation? value)
  (element? OrgContractExpectation value))
(def (org-contract-binding? value) (element? OrgContractBinding value))
(def (org-contract-assertion? value)
  (element? OrgContractAssertion value))
(def (org-contract-definition? value)
  (element? OrgContractDefinition value))
(def (org-contract-result? value) (element? OrgContractResult value))
(def (org-contract-profile? value)
  (element? OrgContractProfile value))
