;;; -*- Gerbil -*-
;;; Test-only serialization of the Scheme-owned AOT Org event algorithm.

(import (only-in :std/encoding/json json->string)
        (only-in "rowan-event-parser.ss" parse-org-rowan-events))
(export rowan-event-fixture rowan-event-fixture-json)

(def (event->json event)
  (case (car event)
    ((start) (vector "start" (symbol->string (cadr event))))
    ((token) (vector "token" (symbol->string (cadr event))
                     (caddr event) (cadddr event)))
    ((finish) (vector "finish"))
    (else (error "unknown Org outline event" event))))

(def (rowan-event-fixture)
  (let (source "#+TODO: TODO | DONE\n* Parent\n#+BEGIN_SRC rust\nα\n#+END_SRC\nsummary\n** Child\nSCHEDULED: <2026-01-01>\nCLOCK: [a]--[b]\n| a | b |\n|---+---|\n| c\\|d | α |\n#+begin_example\n| literal |\n#+end_example\n#+begin_export html\n<b>α</b>\n#+end_export\n#+begin_comment\nignored\n#+end_comment\n:PROPERTIES:\n:ID: alpha\n:EMPTY:\n:END:\nSee [[https://example.test][α]]\n- one\n  continued\n  - two\n\n- three\n#+begin_quote\ntext\n- item\n#+end_quote\n#+BEGIN: note\nmore\n#+END:\n:LOGBOOK:\nentry\n:END:\n")
    (hash (source source)
          (events (list->vector
                   (map event->json (parse-org-rowan-events source)))))))

(def (rowan-event-fixture-json)
  (json->string (rowan-event-fixture) sort-keys: #t))
