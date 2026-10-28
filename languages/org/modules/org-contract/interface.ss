;;; Source selection and admission are owned by Yuanxu.
(.def OrgContract
  identity: 'org-contract
  core: (.o requires: (.o) scope-contract: OrgContractScopeContract
            exports: OrgContractProfile.)
  features: (.o))
