;;; -*- Gerbil -*-
;;; Test-only native values from the Scheme-owned Org event algorithm.

(import (only-in :std/misc/ports read-all-as-string)
        (only-in "event-runtime.ss" parse-org-native-events))
(export event-fixture-source event-fixture)

(def (event-fixture-source)
  (call-with-input-file "languages/org/fixtures/event-source.org"
    read-all-as-string))

(def (event-fixture)
  (parse-org-native-events (event-fixture-source)))
