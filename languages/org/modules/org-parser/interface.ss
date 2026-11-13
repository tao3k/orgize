(import (only-in :tao3k/yuanxu/module-system/declaration .def)
        "./types.ss" "./objects.ss" "./config.ss" "./build.ss")

(.def OrgParser
  identity: 'org-parser
  core: (.o requires: (.o) scope-contract: OrgParserScopeContract
            exports: OrgParser. projections: (list Build))
  features: (.o))
