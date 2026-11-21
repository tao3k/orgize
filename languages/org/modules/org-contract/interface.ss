;;; Source selection and admission are owned by Yuanxu.
(import (only-in :clan/poo/object .def .o)
        (only-in "./types.ss" OrgContractScopeContract)
        (only-in "./config.ss" OrgContractProfile.))

(export OrgContract)

(.def OrgContract
  identity: 'org-contract
  core: (.o scope-contract: OrgContractScopeContract
            exports: OrgContractProfile.)
  features: (.o))
