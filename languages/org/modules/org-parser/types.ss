;;; Yuanxu Module Scope contract; business values have no authoring dependency.
(import (only-in :clan/poo/object .cc .get)
        (only-in :clan/poo/mop define-type Fun Any Bool)
        (only-in :clan/poo/type Record)
        (only-in :yuanxu/object-types/contracts YuanxuContract.))
(export OrgParserScope OrgParserScopeContract)
(define-type OrgParserScope
  (Record event-strategy?: [(Fun Bool <- Any)]))
(def OrgParserScopeContract
  (.cc YuanxuContract. identity: 'org-parser/scope
       value-contract: OrgParserScope proto: (.get OrgParserScope proto)))
