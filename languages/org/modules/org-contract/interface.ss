;;; Source selection and admission are owned by Yuanxu.
(import (only-in :tao3k/yuanxu/module-system/declaration .def)
        "./types.ss" "./objects.ss" "./config.ss")

(.def OrgContract
  identity: 'org-contract
  core: (.o requires: (.o) scope-contract: OrgContractScopeContract
            exports: OrgContractProfile.)
  features: (.o))
