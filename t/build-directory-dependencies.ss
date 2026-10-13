#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Focused consumer preparation in the current package environment.
;;; PackageSpec admits the selected closure; std/make remains its executor.
(import (only-in :clan/poo/object .o)
        (only-in :std/make make)
        (only-in :asp-gerbil-scheme/building-api
                 asp-gerbil-scheme-library-package-prototype
                 asp-gerbil-scheme-package-policy-native-spec))
(export main)
(def parser-profile
  (.o (:: @ asp-gerbil-scheme-library-package-prototype)
      public-entry-modules:
      '("graph-query-support.ss"
        "language-support/grammar.ss"
        "src/language/descriptor.ss"
        "src/modules/parser/interface.ss"
        "src/compiler/event-fold-scheme-modules.ss"
        "src/compiler/event-fold-scheme-build.ss"
        "src/compiler/rust-pure-aot.ss")))
(def (main)
  (let* ((environment (getenv "GERBIL_PATH"))
         (parser (path-expand "pkg/github.com/tao3k/gerbil-parser" environment)))
    (parameterize ((current-directory parser))
      (let (spec (asp-gerbil-scheme-package-policy-native-spec parser-profile))
        (displayln "DIRECTORY-DEPENDENCIES selected=" (length spec))
        (force-output)
        (make spec srcdir: parser libdir: (path-expand "lib" environment)
              optimize: #t parallelize: 4)))
    (displayln "DIRECTORY-DEPENDENCIES-OK")
    (force-output)))
