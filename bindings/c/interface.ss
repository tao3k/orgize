(import (only-in :tao3k/yuanxu/module-system/declaration .def)
        "./types.ss" "./objects.ss" "./config.ss" "./build.ss")

(.def OrgNative
  identity: 'c
  core: (.o requires: (.o) scope-contract: OrgNativeScopeContract
            exports: OrgNative. projections: (list Build))
  features: (.o))
