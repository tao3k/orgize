;;; Source selection and admission are owned by Yuanxu.
(import (only-in :tao3k/yuanxu/module-system/source/declaration .def))

(.def OrgContract
  identity: 'org-contract
  core: (.o requires: (.o) scope-contract: OrgContractScopeContract
            exports: OrgContractProfile.)
  features: (.o))
