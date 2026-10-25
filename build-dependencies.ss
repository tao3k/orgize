#!/usr/bin/env gxi
;;; SDK installation owns revisions, recursion and package builds. Editor TAGS
;;; is an explicit development action, not a compilation/admission requirement.
(import (only-in :gerbil/tools/gxpkg pkg-plist pkg-install with-pkg-build-env))
(export main)
(def (main)
  (def dependencies (pgetq depend: (pkg-plist ".") '()))
  (unless (and (list? dependencies) (andmap string? dependencies))
    (error "expected native package dependency strings" "gerbil.pkg"))
  (with-pkg-build-env
    (for-each pkg-install dependencies))
  (displayln "DEPENDENCIES-READY") (force-output))
