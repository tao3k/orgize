;;; -*- Gerbil -*-
;;; Test-only serialization of the executable Scheme outline algorithm.

(import (only-in :std/encoding/json json->string)
        (only-in "outline-events.ss" parse-org-outline-events))
(export outline-event-fixture-json)

(def (event->json event)
  (case (car event)
    ((start) (vector "start" (symbol->string (cadr event))))
    ((token) (vector "token" (symbol->string (cadr event))
                     (caddr event) (cadddr event)))
    ((finish) (vector "finish"))
    (else (error "unknown Org outline event" event))))

(def (outline-event-fixture-json)
  (let (source "#+TODO: TODO | DONE\n* Parent\n#+BEGIN_SRC rust\nα\n#+END_SRC\nsummary\n** Child\nSCHEDULED: <2026-01-01>\nCLOCK: [a]--[b]\n| a | b |\n|---+---|\n| c\\|d | α |\n#+begin_example\n| literal |\n#+end_example\n#+begin_export html\n<b>α</b>\n#+end_export\n#+begin_comment\nignored\n#+end_comment\n:PROPERTIES:\n:ID: alpha\n:EMPTY:\n:END:\n")
    (json->string
     (hash (source source)
           (events (list->vector
                    (map event->json (parse-org-outline-events source))))))))
