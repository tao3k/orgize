;;; -*- Gerbil -*-
;;; Inert POO values for the Org contract feature.

(import (only-in :clan/poo/object .o .ref)
        (only-in :clan/poo/mop validate)
        (only-in :yuanxu/object-family/interface defpoo-object-family)
        (only-in "value-types.ss"
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
        org-contract-assertion-message org-contract-assertion-fix
        org-contract-definition-id org-contract-definition-scope
        org-contract-definition-assertions
        org-contract-result-assertion-id org-contract-result-matched-count
        org-contract-result-passed?)

(def (make-org-contract-expectation operator-value count-value)
  (validate OrgContractExpectation
          (.o kind: +org-contract-expectation-kind+
              schema: +org-contract-schema+
              operator: operator-value
              count: count-value)))

(def (make-org-contract-binding name-value query-value)
  (validate OrgContractBinding
          (.o kind: +org-contract-binding-kind+
              schema: +org-contract-schema+
              name: name-value
              query: query-value)))

(def (make-org-contract-assertion id-value severity-value query-value
                                  expectation-value (bindings-value '())
                                  (message-value #f) (fix-value #f))
  (validate OrgContractAssertion
          (.o kind: +org-contract-assertion-kind+
              schema: +org-contract-schema+
              id: id-value
              severity: severity-value
              bindings: bindings-value
              query: query-value
              expectation: expectation-value
              message: message-value
              fix: fix-value)))

(def (make-org-contract-definition id-value scope-value assertions-value)
  (validate OrgContractDefinition
          (.o kind: +org-contract-definition-kind+
              schema: +org-contract-schema+
              id: id-value
              scope: scope-value
              assertions: assertions-value)))

(def (make-org-contract-result assertion-id-value matched-count-value
                               passed-value?)
  (validate OrgContractResult
          (.o kind: +org-contract-result-kind+
              schema: +org-contract-schema+
              assertion-id: assertion-id-value
              matched-count: matched-count-value
              passed?: passed-value?)))

(defpoo-object-family
  (accessors
   (org-contract-expectation-operator operator)
   (org-contract-expectation-count count)
   (org-contract-binding-name name)
   (org-contract-binding-query query)
   (org-contract-assertion-id id)
   (org-contract-assertion-severity severity)
   (org-contract-assertion-bindings bindings)
   (org-contract-assertion-query query)
   (org-contract-assertion-expectation expectation)
   (org-contract-assertion-message message)
   (org-contract-assertion-fix fix)
   (org-contract-definition-id id)
   (org-contract-definition-scope scope)
   (org-contract-definition-assertions assertions)
   (org-contract-result-assertion-id assertion-id)
   (org-contract-result-matched-count matched-count)
   (org-contract-result-passed? passed?)
)
  (projections))
