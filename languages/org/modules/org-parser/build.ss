;;; The diagnostic clock needs dynamic lookup for Darwin bundles.
(import (only-in :clan/poo/object .def .o))
(.def Build
  defbuild-script:
  (.o spec: (cond-expand
              (darwin '((gxc: "clock" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
              (else '()))))
