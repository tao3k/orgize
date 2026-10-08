;;; -*- Gerbil -*-
;;; Test-only native values from the Scheme-owned Org event algorithm.

(import (only-in :std/misc/ports read-all-as-string)
        (only-in "rowan-event-runtime.ss" parse-org-rowan-events))
(export rowan-event-fixture-source rowan-event-fixture)

(def (rowan-event-fixture-source)
  (call-with-input-file "languages/org/fixtures/rowan-event-source.org"
    read-all-as-string))

(def (rowan-event-fixture)
  (parse-org-rowan-events (rowan-event-fixture-source)))
