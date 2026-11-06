;;; Only the diagnostic clock needs foreign linker options; roles own source discovery.
(import (only-in :clan/poo/object .o))
(export Build)
(def Build
  (.o kind: 'module/build
      roots: (cond-expand
               (darwin '((gxc: "clock" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
               (else '("clock")))))
