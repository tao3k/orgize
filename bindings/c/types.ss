(import (only-in :clan/poo/object .cc)
        (only-in :clan/poo/mop define-type)
        (only-in :clan/poo/type Record))
(export OrgNativeScope OrgNativeScopeContract)
;; Build-only capability: the native leaves have their own C ABI.
(define-type OrgNativeScope (Record))
(def OrgNativeScopeContract
  (.cc OrgNativeScope identity: 'org-native/scope))
