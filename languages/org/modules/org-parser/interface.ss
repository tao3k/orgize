(import (only-in :tao3k/yuanxu/module-system/source/declaration .def))

(.def OrgParser
  identity: 'org-parser
  core: (.o scope-contract: OrgParserScopeContract
            exports: OrgParser.)
  features: (.o))
