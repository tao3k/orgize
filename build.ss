#!/usr/bin/env gxi

;;; Build the generated package before projecting Orgize's own libraries.
(import (for-syntax
          (only-in :std/source this-source-file)
          (only-in :tao3k/yuanxu/module-system/building/source-package register-module-build-source!)))
(begin-syntax
  (register-module-build-source! (path-directory (this-source-file)) 'orgize))
(import (only-in :std/source this-source-file)
        (only-in :std/misc/process run-process)
        (only-in :clan/poo/object .get .call)
        (only-in :tao3k/yuanxu/module-system/building/source-package register-module-build-source!)
        (only-in :std/make make make-clean)
        (only-in :tao3k/yuanxu/module-system/building/products
                 build-output-directory current-build-source-directory)
        (only-in :gerbil-parser/compiler-build-support
                 initialize-compiler-build! run-build-tool)
        (only-in "./languages/org/modules/org-parser/fold-package.ss"
                 fold-package-source-directory)
        (only-in "./init.ss" orgize.init.exports))
(export main)
(register-module-build-source! (path-directory (this-source-file)) 'orgize)

(def (build-native operation arguments)
  (def projection (.call orgize.init.exports .project-build))
  (def spec (.get projection spec))
  (def root (path-directory (this-source-file)))
  (def output (build-output-directory))
  (case (string->symbol operation)
    ((meta) (write '("spec" "compile" "clean")) (newline))
    ((spec) (.call projection .admit-spec spec) (write spec) (newline))
    ((compile)
     (let (options
           (apply append
             (map (lambda (option)
                    (cond ((equal? option "--optimized") (list build-optimized: #t))
                          ((equal? option "--release") (list build-release: #t))
                          ((equal? option "--debug") (list debug: #t))
                          (else (error "unknown build option" option))))
                  arguments)))
       (.call projection .with-compiler-inputs
         (lambda ()
           (apply make spec srcdir: (or (current-build-source-directory) root)
                  libdir: output options)))))
    ((clean)
     (.call projection .with-clean-outputs
       (lambda ()
         (make-clean spec srcdir: (or (current-build-source-directory) root) libdir: output))))
    (else (error "unknown build operation" operation))))

;; A compiler manifest alone does not admit a loadable dynamic interface.
;; Exercise the installed interface, not a source fallback, before publication.
(def (validate-compiled-tape-interface! output)
  (def interface (path-expand "lib/orgize/bindings/c/event-tape.ssi" output))
  (unless (file-exists? interface)
    (error "compiled event-tape interface is missing" interface))
  (run-build-tool
    (list "env" "DYLD_PRINT_APIS=1" "gxi" "-:max-heap=1G,debug=q" "-e"
      (object->string
        `(begin
           (import (only-in :gerbil-scheme-rust/scheme/runtime gerbil-rs-observe-module-work))
           (gerbil-rs-observe-module-work (lambda () (eval '(import ,interface)))
                                        (current-error-port))
           (displayln "COMPILED-INTERFACE-ABI-OK")
           (force-output))))))

(def (main . arguments)
  (def operation (if (null? arguments) "compile" (car arguments)))
  (def output (gerbil-path))
  (def fold-source (fold-package-source-directory output))
  (def fold-build (path-expand "build.ss" fold-source))
  (initialize-compiler-build!)
  (when (equal? operation "compile")
    ;; Source generation and the package's native build are separate. A
    ;; registered source directory is not an installed dependency.
    (run-build-tool
      (list "gxi" (path-expand "languages/org/modules/org-parser/fold-generator.ss"
                               (path-directory (this-source-file))) output "dynamic"))
    (run-process (cons* "gerbil" "build"
                        (filter (lambda (arg) (member arg '("--optimized" "--release" "--debug")))
                                arguments))
                 directory: fold-source stdout-redirection: #f))
  (begin0 (build-native operation (if (null? arguments) '() (cdr arguments)))
    ;; The bridge compiler owns the program closure and manifest. CI and Cargo
    ;; consume this package product; they do not rebuild a second static subset.
    (when (equal? operation "compile")
      (validate-compiled-tape-interface! output)
      (create-directory* (path-expand "target/gerbil-parser" (path-directory (this-source-file))))
      (run-build-tool
        (list "gxi" (path-expand "bindings/c/stage-parser.ss" (path-directory (this-source-file)))
              (path-expand "target/gerbil-parser" (path-directory (this-source-file))))))
    ;; The generated package's native catalog cleans its own exact targets.
    ;; No grammar, import dispatcher, artifact scanner or timestamp policy here.
    (when (and (equal? operation "clean") (file-exists? fold-build))
      (run-process '("gerbil" "clean") directory: fold-source stdout-redirection: #f))))
