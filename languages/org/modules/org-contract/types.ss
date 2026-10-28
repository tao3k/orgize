;;; Yuanxu Module Scope contract; business values have no authoring dependency.
(import (only-in "value-types.ss" OrgContractProfile))
;; Yuanxu retains the existing profile's value contract.
(import (only-in :clan/poo/object .cc)
        (only-in :yuanxu/object-types/contracts YuanxuContract.))
(export OrgContractScopeContract)
(def OrgContractScopeContract
  (.cc YuanxuContract. identity: 'org-contract/scope
       value-contract: OrgContractProfile))
