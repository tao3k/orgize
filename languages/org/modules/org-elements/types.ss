;;; Yuanxu Module Scope contract; business values have no authoring dependency.
(import (only-in "value-types.ss" OrgElementsProfile)
        (only-in :clan/poo/object .cc))
(export OrgElementsScopeContract)
(def OrgElementsScopeContract
  (.cc OrgElementsProfile identity: 'org-elements/scope))
