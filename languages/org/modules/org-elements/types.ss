;;; Yuanxu Module Scope contract; business values have no authoring dependency.
(import (only-in "value-types.ss" OrgElementsProfile))
;; Yuanxu retains the existing profile's value contract.
(import (only-in :clan/poo/object .cc)
        (only-in :yuanxu/object-types/contracts YuanxuContract.))
(export OrgElementsScopeContract)
(def OrgElementsScopeContract
  (.cc YuanxuContract. identity: 'org-elements/scope
       value-contract: OrgElementsProfile))
