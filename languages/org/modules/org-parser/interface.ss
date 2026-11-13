(import (only-in :tao3k/yuanxu/module-system/source/declaration .def))

(.def OrgParser
  identity: 'org-parser
  core: (.o requires: (.o) scope-contract: OrgParserScopeContract
            exports: OrgParser. projections: (list Build))
  features: (.o))
