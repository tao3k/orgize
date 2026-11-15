#!/usr/bin/env gxi

;;; Orgize package entrypoint for the Yuanxu targets, Fold package and Bridge program.
(import (for-syntax
          (only-in :std/source this-source-file)
          (only-in :tao3k/yuanxu/module-system/building/source-package register-module-build-source!)))
(begin-syntax
  (register-module-build-source! (path-directory (this-source-file)) 'orgize))
(import (only-in :std/source this-source-file)
        (only-in :std/build-script defbuild-script)
        (only-in :clan/poo/object .get .call)
        (only-in :tao3k/yuanxu/module-system/building/source-package register-module-build-source!)
        (only-in :gerbil-parser/compiler-build-support
                 initialize-compiler-build! run-build-tool)
        (only-in "./languages/org/modules/org-parser/fold-package.ss"
                 compile-fold-package! clean-fold-package!)
        (only-in "./init.ss" orgize.init.exports))
(export main)
(register-module-build-source! (path-directory (this-source-file)) 'orgize)

(def (build-targets operation options)
  (def projection (delay (.call orgize.init.exports .project-build)))
  (def (run)
    (defbuild-script (if (equal? operation "meta") '() (.get (force projection) spec)))
    (apply main operation options))
  (case (string->symbol operation)
    ((compile) (.call (force projection) .with-compiler-inputs run))
    ((clean) (.call (force projection) .with-clean-outputs run))
    ((spec)
     (let (build (force projection))
       (.call build .admit-spec (.get build spec)))
     (run))
    (else (run))))

(def (main (operation "compile") . options)
  (def root (path-directory (this-source-file)))
  (def output (gerbil-path))
  (initialize-compiler-build!)
  (when (equal? operation "compile")
    (compile-fold-package! output options))
  (begin0 (build-targets operation options)
    (case (string->symbol operation)
      ((compile)
       (run-build-tool
         (list "gxi" (path-expand "bindings/c/stage-parser.ss" root)
               (path-expand "target/gerbil-parser" root) "installed")))
      ((clean) (clean-fold-package! output)))))
