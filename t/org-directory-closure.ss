;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; Link the existing native gxtest values before the bounded execution gate.
(import (only-in :std/test/base TestHarness TestConfig TestModule test-run! test-result-ok?)
        (only-in "org-directory-test.ss" org-directory-test)
        (only-in "org-contract-test.ss" org-contract-feature-test))
(export main)
(def (main)
  (let (result
        (test-run!
          (TestHarness "Org Directory and Contract"
                       (TestConfig verbosity: 5 capture-output?: #f)
                       (list (TestModule "Org Directory and Contract"
                                         (list org-directory-test org-contract-feature-test)
                                         '() void void)))))
    (if (test-result-ok? result)
      (begin (displayln "OK") (force-output) (exit 0))
      (begin (displayln "FAILED") (force-output) (exit 42)))))
