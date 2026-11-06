(.def OrgNative
  identity: 'c
  core: (.o requires: (.o) scope-contract: OrgNativeScopeContract
            exports: OrgNative. projections: (list Build))
  features: (.o))
