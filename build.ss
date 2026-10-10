#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Orgize owns declaration bootstrap, generated parser products and runtime.
(module runtime-build
  (export main)
  (import (only-in :std/build-script defbuild-script)
          (only-in :std/source this-source-file))

  ;; Gambit compiles generated C in GERBIL_PATH, not beside the source header.
  ;; The include search path is scoped to this build process and retains an
  ;; existing caller path (for example one supplied by Nix).
  (let ((header-root (path-expand "bindings/c/include" (current-directory)))
        (previous (getenv "C_INCLUDE_PATH" #f)))
    (setenv "C_INCLUDE_PATH"
            (if (and previous (not (equal? previous "")))
              (string-append header-root ":" previous)
              header-root)))

  (defbuild-script
   '((gxc: "languages/org/modules/org-elements/graph-types.ss")
     (gxc: "languages/org/modules/org-elements/graph-objects.ss")
     (gxc: "languages/org/graph-shape.ss")
     (gxc: "languages/org/modules/org-elements/types.ss")
     (gxc: "languages/org/modules/org-elements/objects.ss")
     (gxc: "languages/org/modules/org-elements/funs.ss")
     (gxc: "languages/org/modules/org-elements/syntax.ss")
     (gxc: "languages/org/modules/org-elements/runtime-interface.ss")
     (gxc: "languages/org/event-runtime.ss")
     (gxc: "languages/org/modules/org-elements/headline-properties.ss")
     (gxc: "languages/org/modules/org-elements/source-headlines.ss")
     (gxc: "languages/org/modules/org-elements/source-interface.ss")
     (gxc: "languages/org/modules/org-contract/types.ss")
     (gxc: "languages/org/modules/org-contract/objects.ss")
     (gxc: "languages/org/modules/org-contract/funs.ss")
     (gxc: "languages/org/modules/org-contract/config.ss")
     (gxc: "languages/org/modules/org-contract/syntax.ss")
     (gxc: "languages/org/modules/org-contract/runtime-interface.ss")
     (gxc: "bindings/c/orgize.ss"))))

(import (only-in :std/misc/process run-process)
        (prefix-in runtime-build runtime-))
(export main)

(def (main . arguments)
  (when (or (null? arguments) (equal? (car arguments) "compile"))
    (let (previous (getenv "ORGIZE_BUILD_SYNTAX_ONLY" #f))
      (dynamic-wind
        (lambda () (setenv "ORGIZE_BUILD_SYNTAX_ONLY" "1"))
        (lambda ()
          (run-process
            (cons "gxi" (cons "build-parser.ss"
                             (if (null? arguments) '("compile") arguments)))
            stdin-redirection: #f stdout-redirection: #f))
        (lambda ()
          (if previous
            (setenv "ORGIZE_BUILD_SYNTAX_ONLY" previous)
            (setenv "ORGIZE_BUILD_SYNTAX_ONLY")))))
    (run-process
      (list "gxi" "languages/org/generate-fold.ss"
            (getenv "GERBIL_PATH" (path-expand "~/.gerbil")) "dynamic")
      stdin-redirection: #f stdout-redirection: #f))
  (apply runtime-main arguments))
