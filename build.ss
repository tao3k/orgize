#!/usr/bin/env -S timeout --signal=TERM --kill-after=5s 600s gxi
;;; -*- Gerbil -*-
;;; Orgize owns declaration bootstrap, generated parser products and runtime.
;;; Complete cold compilation includes generated parsers on small CI runners.
;;; This build-only envelope does not change native runtime/test watchdogs.
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
     (gxc: "languages/org/modules/org-elements/source-graph.ss")
     (gxc: "languages/org/modules/org-elements/source-interface.ss")
     (gxc: "languages/org/modules/org-contract/types.ss")
     (gxc: "languages/org/modules/org-contract/objects.ss")
     (gxc: "languages/org/modules/org-contract/funs.ss")
     (gxc: "languages/org/modules/org-contract/config.ss")
     (gxc: "languages/org/modules/org-contract/syntax.ss")
     (gxc: "languages/org/modules/org-contract/runtime-interface.ss")
     (gxc: "languages/org/modules/org-contract/directory.ss")
     (gxc: "languages/org/modules/org-contract/source.ss")
     (gxc: "languages/org/modules/org-contract/directory-runtime.ss")
     (gxc: "languages/org/modules/org-contract/interface.ss")
     (gxc: "bindings/c/orgize.ss"))))

(import (only-in :std/source this-source-file)
        (only-in :gerbil/expander current-expander-context make-top-context eval-syntax)
        (only-in :gerbil-parser/src/build-support/compiler-profile initialize-compiler-build!)
        (prefix-in runtime-build runtime-))
(export main)

;; Reuse the SDK compiler/module cache across this package's ordered phases.
;; Metadata requests remain inert: no grammar or generator is loaded here.
(def build-root (path-directory (this-source-file)))
(def (script-main relative)
  (parameterize ((current-expander-context (make-top-context)))
    (eval-syntax '(import :gerbil/core))
    (eval-syntax `(import (only-in ,(path-expand relative build-root) main)))
    (eval-syntax 'main)))
(def parser-builder (delay (script-main "build-parser.ss")))
(def fold-builder (delay (script-main "languages/org/generate-fold.ss")))

(def (main . arguments)
  (def operation (if (null? arguments) "compile" (car arguments)))
  (def active? (and (member operation '("compile" "clean")) #t))
  (def started (##current-time-point))
  (def (phase name)
    ;; Actual entry transitions only; metadata/spec output stays SDK-owned.
    (when active?
      (displayln "PACKAGE-BUILD-PHASE orgize operation=" operation " stage=" name
                 " elapsed-s=" (- (##current-time-point) started))
      (force-output)))
  ;; A dependency child cannot initialize this package process's host policy.
  (phase 'compiler-policy-enter)
  (initialize-compiler-build!)
  (phase 'compiler-policy-returned)
  (when active?
    (phase 'syntax-enter)
    (let (previous (getenv "ORGIZE_BUILD_SYNTAX_ONLY" #f))
      (dynamic-wind
        (lambda () (setenv "ORGIZE_BUILD_SYNTAX_ONLY" "1"))
        (lambda ()
          (apply (force parser-builder)
                 (if (null? arguments) '("compile") arguments)))
        (lambda ()
          (if previous
            (setenv "ORGIZE_BUILD_SYNTAX_ONLY" previous)
            (setenv "ORGIZE_BUILD_SYNTAX_ONLY")))))
    (phase 'syntax-returned)
    (phase 'fold-enter)
    ((force fold-builder)
      (getenv "GERBIL_PATH" (path-expand "~/.gerbil")) "dynamic"
      (if (equal? operation "clean") "clean" "compile"))
    (phase 'fold-returned))
  (phase 'runtime-enter)
  (begin0 (apply runtime-main arguments) (phase 'runtime-returned)))
