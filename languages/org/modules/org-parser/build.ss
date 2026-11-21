;;; The diagnostic clock needs dynamic lookup for Darwin bundles.
(import (only-in :clan/poo/object .def .o))
(import (only-in :gerbil-parser/build-support/compiler initialize-compiler-build!)
        (only-in "./fold-package.ss" compile-fold-package! clean-fold-package!))
(export Build)

(.def Build
  .prepare:
  (lambda (options)
    (initialize-compiler-build!)
    (compile-fold-package! (gerbil-path) options))
  .clean:
  (lambda () (clean-fold-package! (gerbil-path)))
  defbuild-script:
  (.o spec: (cond-expand
              (darwin '((gxc: "clock" "-ld-options" "-Wl,-undefined,dynamic_lookup")))
              (else '()))))
