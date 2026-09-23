;;; -*- Gerbil -*-
;;; Inert POO values for the Org contract feature.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop element?)
        (only-in "types.ss"
                 +org-contract-schema+
                 +org-contract-expectation-kind+
                 +org-contract-binding-kind+ +org-contract-assertion-kind+
                 +org-contract-definition-kind+ +org-contract-result-kind+
                 OrgContractExpectation OrgContractBinding
                 OrgContractAssertion OrgContractDefinition OrgContractResult))
(export make-org-contract-expectation
        make-org-contract-binding make-org-contract-assertion
        make-org-contract-definition make-org-contract-result
        org-contract-expectation-operator org-contract-expectation-count
        org-contract-binding-name org-contract-binding-query
        org-contract-assertion-id org-contract-assertion-severity
        org-contract-assertion-bindings
        org-contract-assertion-query org-contract-assertion-expectation
        org-contract-definition-id org-contract-definition-scope
        org-contract-definition-assertions
        org-contract-result-assertion-id org-contract-result-matched-count
        org-contract-result-passed?)

(def (admit! type value)
  (unless (element? type value)
    (error "invalid Org contract POO value" value))
  value)

(def (make-org-contract-expectation operator-value count-value)
  (admit! OrgContractExpectation
          (.o kind: +org-contract-expectation-kind+
              schema: +org-contract-schema+
              operator: operator-value
              count: count-value)))

(def (make-org-contract-binding name-value query-value)
  (admit! OrgContractBinding
          (.o kind: +org-contract-binding-kind+
              schema: +org-contract-schema+
              name: name-value
              query: query-value)))

(def (make-org-contract-assertion id-value severity-value query-value
                                  expectation-value (bindings-value '()))
  (admit! OrgContractAssertion
          (.o kind: +org-contract-assertion-kind+
              schema: +org-contract-schema+
              id: id-value
              severity: severity-value
              bindings: bindings-value
              query: query-value
              expectation: expectation-value)))

(def (make-org-contract-definition id-value scope-value assertions-value)
  (admit! OrgContractDefinition
          (.o kind: +org-contract-definition-kind+
              schema: +org-contract-schema+
              id: id-value
              scope: scope-value
              assertions: assertions-value)))

(def (make-org-contract-result assertion-id-value matched-count-value
                               passed-value?)
  (admit! OrgContractResult
          (.o kind: +org-contract-result-kind+
              schema: +org-contract-schema+
              assertion-id: assertion-id-value
              matched-count: matched-count-value
              passed?: passed-value?)))

(def (org-contract-expectation-operator value) (.ref value 'operator))
(def (org-contract-expectation-count value) (.ref value 'count))
(def (org-contract-binding-name value) (.ref value 'name))
(def (org-contract-binding-query value) (.ref value 'query))
(def (org-contract-assertion-id value) (.ref value 'id))
(def (org-contract-assertion-severity value) (.ref value 'severity))
(def (org-contract-assertion-bindings value) (.ref value 'bindings))
(def (org-contract-assertion-query value) (.ref value 'query))
(def (org-contract-assertion-expectation value) (.ref value 'expectation))
(def (org-contract-definition-id value) (.ref value 'id))
(def (org-contract-definition-scope value) (.ref value 'scope))
(def (org-contract-definition-assertions value) (.ref value 'assertions))
(def (org-contract-result-assertion-id value) (.ref value 'assertion-id))
(def (org-contract-result-matched-count value) (.ref value 'matched-count))
(def (org-contract-result-passed? value) (.ref value 'passed?))
