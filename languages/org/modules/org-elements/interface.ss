;;; Source selection and admission are owned by Yuanxu.
(.def OrgElements
  identity: 'org-elements
  core: (.o requires: (.o) scope-contract: OrgElementsScopeContract
            exports: OrgElementsProfile.)
  features: (.o))
