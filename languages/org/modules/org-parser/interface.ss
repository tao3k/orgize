(import (only-in :clan/poo/object .def .o)
        (only-in "./types.ss" OrgParserScopeContract)
        (only-in "./objects.ss" OrgParser.))

(export OrgParser)

(.def OrgParser
  identity: 'org-parser
  core: (.o scope-contract: OrgParserScopeContract
            exports: OrgParser.)
  features: (.o))
