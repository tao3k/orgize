#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Select only the native Contract test closure; no Rust bridge is required.
(import (only-in :clan/poo/object .o)
        (only-in :std/make make)
        (only-in :asp-gerbil-scheme/building-api
                 asp-gerbil-scheme-library-package-prototype
                 asp-gerbil-scheme-package-policy-native-spec))
(export main)
(def profile
  (.o (:: @ asp-gerbil-scheme-library-package-prototype)
      public-entry-modules: '("t/org-directory-test.ss" "t/org-contract-test.ss"
                               "t/org-elements-module-test.ss")))
(def (main)
  (let (spec (asp-gerbil-scheme-package-policy-native-spec profile))
    (make spec srcdir: (current-directory)
          libdir: (path-expand "lib" (getenv "GERBIL_PATH"))
          optimize: #t parallelize: 4)
    (displayln "DIRECTORY-NATIVE-CLOSURE-PREPARED")
    (force-output)))
