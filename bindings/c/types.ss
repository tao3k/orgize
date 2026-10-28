(import (only-in :clan/poo/object .cc .get)
        (only-in :clan/poo/mop define-type)
        (only-in :clan/poo/type Record)
        (only-in :yuanxu/object-types/contracts YuanxuContract.))
(export OrgNativeScope OrgNativeScopeContract)
;; Build-only capability: the native leaves have their own C ABI.
(define-type OrgNativeScope (Record))
(def OrgNativeScopeContract
  (.cc YuanxuContract. identity: 'org-native/scope
       value-contract: OrgNativeScope proto: (.get OrgNativeScope proto)))
