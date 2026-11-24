;;; Actual foreign leaves retain their platform compiler options.
(import (only-in :clan/poo/object .def .o)
        (only-in :std/source this-source-file)
        (only-in :gerbil-parser/build-support/compiler run-build-tool))
(export Build)

(.def Build
  .finish:
  (lambda ()
    (def root (path-expand "../.." (path-directory (this-source-file))))
    (run-build-tool
      (list "gxi" (path-expand "bindings/c/stage-parser.ss" root)
            (path-expand "target/gerbil-parser" root) "installed")))
  defbuild-script:
  (.o spec:
    (let ((include-option (string-append "-I" (path-expand "include" (path-directory (this-source-file)))))
          (link-options (cond-expand
                          (darwin '("-ld-options" "-Wl,-undefined,dynamic_lookup"))
                          (else '()))))
      (cons `(gxc: "orgize-parser" ,@link-options)
            (map (lambda (name)
                   `(gxc: ,name "-cc-options" ,include-option ,@link-options))
                 '("orgize" "orgize-scheme-runtime"))))))
