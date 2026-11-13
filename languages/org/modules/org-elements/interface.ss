;;; Source selection and admission are owned by Yuanxu.
(import (only-in :tao3k/yuanxu/module-system/declaration .def)
        "./types.ss" "./objects.ss" "./config.ss")

(.def OrgElements
  identity: 'org-elements
  core: (.o requires: (.o) scope-contract: OrgElementsScopeContract
            exports: OrgElementsProfile.)
  features: (.o))
