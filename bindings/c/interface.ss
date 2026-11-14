(import (only-in :tao3k/yuanxu/module-system/source/declaration .def))

(.def OrgNative
  identity: 'c
  core: (.o scope-contract: OrgNativeScopeContract
            exports: OrgNative.)
  features: (.o))
