;;; -*- Gerbil -*-
;;; Test-only native values from the Scheme-owned Org event algorithm.

(import (only-in :std/misc/ports read-all-as-string)
        (only-in "native-event-runtime.ss" parse-org-native-events))
(export native-event-fixture-source native-event-fixture)

(def (native-event-fixture-source)
  (call-with-input-file "languages/org/fixtures/native-event-source.org"
    read-all-as-string))

(def (native-event-fixture)
  (parse-org-native-events (native-event-fixture-source)))
