#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Compile the runtime and bounded source-headline interface. Native imports
;;; close the current runtime graph; generator tooling stays in its own lane.

(import (only-in :std/build-script defbuild-script))

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
   (gxc: "languages/org/modules/org-elements/source-headlines.ss")
   (gxc: "languages/org/modules/org-elements/source-interface.ss")
   (gxc: "languages/org/modules/org-contract/types.ss")
   (gxc: "languages/org/modules/org-contract/objects.ss")
   (gxc: "languages/org/modules/org-contract/funs.ss")
   (gxc: "languages/org/modules/org-contract/config.ss")
   (gxc: "languages/org/modules/org-contract/syntax.ss")
   (gxc: "languages/org/modules/org-contract/runtime-interface.ss")
   (gxc: "bindings/c/orgize-native.ss")))
