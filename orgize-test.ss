;;; Native gerbil test discovers these standard exported Suites once.
(import (only-in :std/io/tempfile make-temporary-file-name clear-temporary-file!)
        (only-in "t/org-publishing-value-test.ss" org-publishing-value-test)
        (only-in "t/org-citation-export-test.ss" org-citation-export-test)
        (only-in "t/org-metadata-value-test.ss" org-metadata-value-test)
        (only-in "t/org-source-value-test.ss" org-source-value-test)
        (only-in "t/org-lifecycle-value-test.ss" org-lifecycle-value-test)
        (only-in "t/org-semantic-owner-test.ss" org-semantic-owner-test)
        (only-in "t/org-event-tape-test.ss" org-event-tape-test)
        (only-in "t/org-elements-module-test.ss" org-elements-module-test)
        (only-in "t/org-radio-match-test.ss" org-radio-match-test)
        (only-in "t/org-inline-parser-test.ss" org-inline-parser-test)
        (only-in "t/org-structural-parser-test.ss" org-structural-parser-test)
        (only-in "t/org-table-container-parser-test.ss" org-table-container-parser-test)
        (only-in "t/org-source-headlines-qualification.ss" org-source-headlines-test)
        (only-in "t/org-directory-test.ss" org-directory-test)
        (only-in "t/org-contract-test.ss" org-contract-feature-test)
        (only-in "t/org-elements-graph-types-test.ss" org-elements-graph-types-test)
        (only-in "t/org-graph-test.ss" org-graph-test)
        (only-in "t/org-module-boundary-test.ss" org-module-boundary-test)
)
(export test-setup! test-cleanup!
        org-publishing-value-test
        org-citation-export-test
        org-metadata-value-test
        org-source-value-test
        org-lifecycle-value-test
        org-semantic-owner-test
        org-event-tape-test
        org-elements-module-test
        org-radio-match-test
        org-inline-parser-test
        org-structural-parser-test
        org-table-container-parser-test
        org-source-headlines-test
        org-directory-test
        org-contract-feature-test
        org-elements-graph-types-test
        org-graph-test
        org-module-boundary-test
)
(def fixture-root #f)
(def previous-fixture-root #f)
(def (test-setup!)
  (set! previous-fixture-root (getenv "ORGIZE_DIRECTORY_TEST_ROOT" #f))
  (set! fixture-root (make-temporary-file-name "orgize-cli-"))
  (create-directory fixture-root)
  (setenv "ORGIZE_DIRECTORY_TEST_ROOT" fixture-root))
(def (test-cleanup!)
  (when fixture-root (clear-temporary-file! fixture-root))
  (if previous-fixture-root
    (setenv "ORGIZE_DIRECTORY_TEST_ROOT" previous-fixture-root)
    (setenv "ORGIZE_DIRECTORY_TEST_ROOT")))
