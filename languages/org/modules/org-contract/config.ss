;;; -*- Gerbil -*-
;;; Maintained Org-native contract profile; no expression-language mode.

(import (only-in :clan/poo/object .o)
        (only-in "types.ss"
                 +org-contract-schema+ +org-contract-profile-kind+
                 org-contract-profile?))
(export OrgContractProfile. org-contract-default-profile)

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
