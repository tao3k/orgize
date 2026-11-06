;;; Yuanxu Module Scope contract; business values have no authoring dependency.
(import (only-in "value-types.ss" OrgContractProfile)
        (only-in :clan/poo/object .cc))
(export OrgContractScopeContract)
(def OrgContractScopeContract
  (.cc OrgContractProfile identity: 'org-contract/scope))
