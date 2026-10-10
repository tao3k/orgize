;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; AOT entry point for the existing Gerbil test suites; no runtime expansion.
(import (only-in :std/test/base TestHarness TestConfig TestModule
                 test-run! test-result-ok?)
        (only-in :std/test test-suite test-case check)
        "org-publishing-value-test.ss"
        "org-citation-export-test.ss"
        "org-metadata-value-test.ss"
        "org-source-value-test.ss"
        "org-lifecycle-value-test.ss"
        "org-semantic-owner-test.ss"
        "org-elements-module-test.ss"
        "org-radio-match-test.ss"
        "org-event-parser-test.ss"
        "org-source-headlines-qualification.ss")
(export main)

(def (main . args)
  (unless (or (null? args) (equal? args '("--failure-control")))
    (error "native closure accepts no filters" args))
  (let* ((suites (list org-publishing-value-test
                       org-citation-export-test
                       org-metadata-value-test
                       org-source-value-test
                       org-lifecycle-value-test
                       org-semantic-owner-test
                       org-elements-module-test org-radio-match-test
                       org-inline-parser-test
                       org-structural-parser-test
                       org-table-container-parser-test
                       org-source-headlines-test))
         (selected (if (null? args) suites
                     (list (test-suite "failure exit control"
                             (test-case "intentional failed assertion"
                               (check #f => #t))))))
         (modules (list (TestModule "native Org closure" selected '() void void)))
         (result (test-run! (TestHarness "native Org closure"
                                        (TestConfig verbosity: 5 capture-output?: #f)
                                        modules))))
    (if (test-result-ok? result)
      (begin (displayln "OK") (force-output) (exit 0))
      (begin (displayln "FAILED") (force-output) (exit 42)))))
