;;; -*- Gerbil -*-
;;; SPDX-FileCopyrightText: 2026 tao3k team and Contributors
;;; SPDX-License-Identifier: Apache-2.0 AND LGPL-2.1-or-later
;;; AOT entry point for the existing Gerbil test suites; no runtime expansion.
(import (only-in :std/test/base TestHarness TestConfig TestModule
                 test-run! test-result-ok?)
        (only-in :std/test test-suite test-case check)
        "org-native-publishing-value-test.ss"
        "org-native-citation-export-test.ss"
        "org-native-metadata-value-test.ss"
        "org-native-source-value-test.ss"
        "org-native-lifecycle-value-test.ss"
        "org-native-semantic-owner-test.ss"
        "org-elements-module-test.ss"
        "org-radio-match-test.ss"
        "org-rowan-event-parser-test.ss"
        "org-source-headlines-qualification.ss")
(export main)

(def (main . args)
  (unless (or (null? args) (equal? args '("--failure-control")))
    (error "native closure accepts no filters" args))
  (let* ((suites (list org-native-publishing-value-test
                       org-native-citation-export-test
                       org-native-metadata-value-test
                       org-native-source-value-test
                       org-native-lifecycle-value-test
                       org-native-semantic-owner-test
                       org-elements-module-test org-radio-match-test
                       org-v1-rowan-inline-parser-test
                       org-v1-rowan-structural-parser-test
                       org-v1-rowan-table-container-parser-test
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
