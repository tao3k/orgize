;;; -*- Gerbil -*-
;;; Scheme-owned recognition of lifecycle line shapes inside LOGBOOK drawers.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-prefix?))
(export logbook-line-kind logbook-line-kind-rust)

;; The caller supplies one trimmed LOGBOOK line without its optional list dash.
;; Value extraction remains a separate semantic projection over that line.
(define-rust-pure logbook-line-kind logbook-line-kind-rust
  ((line "&str")) "&str"
  (if (string-prefix? line "State ") "state"
      (if (string-prefix? line "Note taken on") "note"
          (if (or (string-prefix? line "Refiled")
                  (string-prefix? line "Refiling")) "refile"
              (if (string-prefix? line "Rescheduled") "reschedule"
                  (if (or (string-prefix? line "New deadline")
                          (string-prefix? line "Deadline")
                          (string-prefix? line "Removed deadline")) "redeadline"
                      (if (string-prefix? line "CLOCK:") "clock" "note")))))))
