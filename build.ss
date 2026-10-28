#!/usr/bin/env gxi

;;; Org generation precedes the one Yuanxu-owned native library projection.
(import (for-syntax
          (only-in :std/source this-source-file)
          (only-in :yuanxu/module-system/building/source-package register-module-build-source!)))
(begin-syntax
  (register-module-build-source! (path-directory (this-source-file)) 'orgize))
(import (only-in :std/source this-source-file)
        (only-in :clan/poo/object .get)
        (only-in :yuanxu/module-system/building/source-package register-module-build-source!)
        (only-in :yuanxu/module-system/building/script defbuild-script)
        (only-in :gerbil-parser/src/build-support/compiler-profile
                 initialize-compiler-build! run-build-tool)
        (only-in "./init.ss" orgize.init.exports))
(export main)
(register-module-build-source! (path-directory (this-source-file)) 'orgize)

(def (build-native . arguments)
  (def modules-spec (.get orgize.init.exports std-make-spec))
  (defbuild-script (modules-spec '())
    parallelize: (min 2 (string->number (getenv "GERBIL_BUILD_CORES" "0"))))
  (apply main arguments))

(def (main . arguments)
  (def operation (if (null? arguments) "compile" (car arguments)))
  (def output (gerbil-path))
  (initialize-compiler-build!)
  (when (equal? operation "compile")
    ;; The native producer process owns grammar expansion and emission. Keep
    ;; that compiler heap out of the subsequent Yuanxu library projection.
    (run-build-tool
      (list "gxi" (path-expand "languages/org/modules/org-parser/fold-generator.ss"
                               (path-directory (this-source-file))) output "dynamic")))
  (when (file-exists? (path-expand "gerbil.pkg" output))
    (register-module-build-source! output 'orgize-fold))
  (begin0 (apply build-native arguments)
    ;; The bridge compiler owns the program closure and manifest. CI and Cargo
    ;; consume this package product; they do not rebuild a second static subset.
    (when (equal? operation "compile")
      (create-directory* (path-expand "target/gerbil-parser" (path-directory (this-source-file))))
      (run-build-tool
        (list "gxi" (path-expand "bindings/c/stage-parser.ss" (path-directory (this-source-file)))
              (path-expand "target/gerbil-parser" (path-directory (this-source-file))))))
    ;; The generated package's native catalog cleans its own exact targets.
    ;; No grammar, import dispatcher, artifact scanner or timestamp policy here.
    (when (and (equal? operation "clean")
               (file-exists? (path-expand "build.ss" output)))
      (run-build-tool (list "gxi" (path-expand "build.ss" output) "clean")))))
