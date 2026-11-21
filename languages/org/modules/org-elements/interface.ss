;;; Source selection and admission are owned by Yuanxu.
(import (only-in :clan/poo/object .def .o)
        (only-in "./types.ss" OrgElementsScopeContract)
        (only-in "./config.ss" OrgElementsProfile.))

(export OrgElements)

(.def OrgElements
  identity: 'org-elements
  core: (.o scope-contract: OrgElementsScopeContract
            exports: OrgElementsProfile.)
  features: (.o))
