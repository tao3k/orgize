;;; Yuanxu Module Scope contract; business values have no authoring dependency.
(import (only-in :clan/poo/object .cc)
        (only-in :clan/poo/mop define-type Fun Any Bool)
        (only-in :clan/poo/type Record))
(export OrgParserScope OrgParserScopeContract)
(define-type OrgParserScope
  (Record event-strategy?: [(Fun Bool <- Any)]))
(def OrgParserScopeContract
  (.cc OrgParserScope identity: 'org-parser/scope))
