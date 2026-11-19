#!/usr/bin/env gxi
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later

(import (only-in :clan/poo/object .call .get)
        (only-in :std/build-script defbuild-script)
        (only-in "./package-spec.ss" orgize.package.exports))
(export main)

(def (main . arguments)
  (def (build)
    (defbuild-script (.get orgize.package.exports std-make-spec))
    (apply main arguments))
  (def actions (.get orgize.package.exports build-actions))
  (match arguments
    ((or [] ["compile" . _])
     ;; The generated Fold interfaces are compiler inputs.
     (.call actions .prepare (if (pair? arguments) (cdr arguments) '()))
     (build)
     (.call actions .finish))
    (["clean"]
     (build)
     (.call actions .clean))
    (else (build))))
