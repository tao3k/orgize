;;; Actual foreign leaves retain their platform compiler options.
(import (only-in :clan/poo/object .o)
        (only-in :std/source this-source-file))
(export Build)
(def Build
  (let ((include-option (string-append "-I" (path-expand "include" (path-directory (this-source-file)))))
        (link-options (cond-expand
                        (darwin '("-ld-options" "-Wl,-undefined,dynamic_lookup"))
                        (else '()))))
    (.o kind: 'module/build
        roots: (append
                 (map (lambda (name) `(gxc: ,name ,@link-options))
                      '("orgize-parser"))
                 (map (lambda (name)
                        `(gxc: ,name "-cc-options" ,include-option ,@link-options))
                      '("orgize" "orgize-scheme-runtime"))))))
