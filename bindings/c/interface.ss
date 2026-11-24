(import (only-in :clan/poo/object .def .o)
        (only-in "./types.ss" OrgNativeScopeContract)
        (only-in "./objects.ss" OrgNative.))

(export OrgNative)

(.def OrgNative
  identity: 'c
  core: (.o scope-contract: OrgNativeScopeContract
            exports: OrgNative.)
  features: (.o))
