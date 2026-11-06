;;; Org contracts use native Record field admission and MOP validation.
(import (only-in :clan/poo/mop define-type Type. element? Bool)
        (only-in :clan/poo/type Record Exactly Enum String List OrFalse)
        (only-in :clan/poo/number UInt)
        (only-in :orgize/languages/org/modules/org-elements/value-types OrgElementQuery))
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
        org-contract-profile? OrgDirectoryProfile org-directory-profile?)

(def +org-contract-schema+ "orgize.org-contract.v1")
(def +org-contract-expectation-kind+ 'org-contract-expectation)
(def +org-contract-binding-kind+ 'org-contract-binding)
(def +org-contract-assertion-kind+ 'org-contract-assertion)
(def +org-contract-definition-kind+ 'org-contract-definition)
(def +org-contract-result-kind+ 'org-contract-result)
(def +org-contract-profile-kind+ 'org-contract-profile)


(define-type (ContractName @ Type.)
  .element?: (lambda (v) (and (string? v) (> (string-length v) 0))))

(define-type OrgContractExpectation
  (Record kind: [(Exactly +org-contract-expectation-kind+)]
          schema: [(Exactly +org-contract-schema+)]
          operator: [(Enum at-least exactly at-most)] count: [UInt]))
(define-type OrgContractBinding
  (Record kind: [(Exactly +org-contract-binding-kind+)]
          schema: [(Exactly +org-contract-schema+)]
          name: [ContractName] query: [OrgElementQuery]))
(define-type OrgContractAssertion
  (Record kind: [(Exactly +org-contract-assertion-kind+)]
          schema: [(Exactly +org-contract-schema+)]
          id: [ContractName] severity: [(Enum error warning info)]
          bindings: [(List OrgContractBinding)] query: [OrgElementQuery]
          expectation: [OrgContractExpectation]
          message: [(OrFalse String)] fix: [(OrFalse String)]))
(define-type OrgContractDefinition
  (Record kind: [(Exactly +org-contract-definition-kind+)]
          schema: [(Exactly +org-contract-schema+)]
          id: [ContractName] scope: [(Enum document subtree)]
          assertions: [(List OrgContractAssertion)]))
(define-type OrgContractResult
  (Record kind: [(Exactly +org-contract-result-kind+)]
          schema: [(Exactly +org-contract-schema+)]
          assertion-id: [ContractName] matched-count: [UInt] passed?: [Bool]))
(define-type OrgContractProfile
  (Record kind: [(Exactly +org-contract-profile-kind+)]
          schema: [(Exactly +org-contract-schema+)]
          source-form: [(Exactly 'org-headings-and-properties)]
          aot-target: [(Exactly 'rust)] runtime-owner: [(Exactly "orgize")]))
(define-type CommonDirectoryContracts
  (Record directory: [(OrFalse ContractName)]
          definitions: [(List OrgContractDefinition) optional: #t]))
(define-type OrgDirectoryProfile
  (Record kind: [(Exactly 'org-directory-profile)]
          schema: [(Exactly "orgize.directory-profile.v1")]
          workspace: [(OrFalse ContractName)] directory: [ContractName]
          builtin: [(List OrgContractDefinition)] local: [(List OrgContractDefinition)]
          contracts: [CommonDirectoryContracts]))

(def (org-directory-profile? v) (element? OrgDirectoryProfile v))
(def (org-contract-expectation? v) (element? OrgContractExpectation v))
(def (org-contract-binding? v) (element? OrgContractBinding v))
(def (org-contract-assertion? v) (element? OrgContractAssertion v))
(def (org-contract-definition? v) (element? OrgContractDefinition v))
(def (org-contract-result? v) (element? OrgContractResult v))
(def (org-contract-profile? v) (element? OrgContractProfile v))
