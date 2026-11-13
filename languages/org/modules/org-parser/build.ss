;;; The diagnostic clock needs dynamic lookup for Darwin bundles.
(export Build)
(def Build
  (cond-expand
    (darwin '((gxc: "clock" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
    (else '())))
