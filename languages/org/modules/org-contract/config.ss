;;; -*- Gerbil -*-
;;; Maintained Org-native contract profile; no expression-language mode.

(import (only-in :clan/poo/object .o)
        (only-in "types.ss"
                 +org-contract-schema+ +org-contract-profile-kind+
                 org-contract-profile?))
(export OrgContractProfile. org-contract-default-profile OrgDirectoryProfile.)

;; One downstream declaration refines this native prototype. Common Contract
;; DI is nested; the kernel retains its mandatory baseline independently.
(def OrgDirectoryProfile.
  (.o kind: 'org-directory-profile schema: "orgize.directory-profile.v1"
      workspace: #f directory: #f builtin: '()
      contracts: (.o directory: #f definitions: '()) local: '()))

(def OrgContractProfile.
  (.o kind: +org-contract-profile-kind+
      schema: +org-contract-schema+
      source-form: 'org-headings-and-properties
      aot-target: 'rust
      runtime-owner: "orgize"))

(def org-contract-default-profile
  (begin
    (unless (org-contract-profile? OrgContractProfile.)
      (error "invalid maintained Org contract profile"))
    OrgContractProfile.))
