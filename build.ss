#!/usr/bin/env gxi
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
(import (only-in :clan/poo/object .call .get)
        (only-in :std/source this-source-file)
        (only-in :std/build-script defbuild-script)
        (rename-in (only-in "./languages/org/modules/org-parser/build.ss" Build) (Build FoldBuild))
        (rename-in (only-in "./bindings/c/build.ss" Build) (Build ProgramBuild))
        (only-in "./init.ss" orgize.init.exports))
(export main)

(defvalues (main)
  (let ()
    (defbuild-script (.get orgize.init.exports std-make-spec))
    (def native-main main)
    (lambda args
      (match args
        ((or [] ["compile" . _])
         (.call FoldBuild .prepare (if (pair? args) (cdr args) '()))
         (apply native-main args)
         (.call ProgramBuild .finish))
        (["clean"]
         (native-main "clean")
         (.call FoldBuild .clean))
        (else (apply native-main args))))))
