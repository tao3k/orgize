;;; Source selection and admission are owned by Yuanxu.
(import (only-in :tao3k/yuanxu/module-system/source/declaration .def))

(.def OrgElements
  identity: 'org-elements
  core: (.o scope-contract: OrgElementsScopeContract
            exports: OrgElementsProfile.)
  features: (.o))
