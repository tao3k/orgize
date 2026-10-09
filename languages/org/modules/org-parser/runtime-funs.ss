;;; -*- Gerbil -*-
;;; Effects stay inside the existing event-fold runtime owner.
(import (only-in :gerbil-parser/src/compiler/event-fold-runtime run-event-fold-program)
        (only-in "objects.ss"
                 org-event-strategy-program))
(export run-org-event-strategy)

(def (run-org-event-strategy strategy source (overrides '()))
  (run-event-fold-program (org-event-strategy-program strategy) source overrides))
