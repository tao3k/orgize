;;; -*- Gerbil -*-
;;; Effects stay inside the existing event-fold runtime owner.
(import (only-in :gerbil-parser/src/compiler/event-fold-runtime run-event-fold)
        (only-in "objects.ss"
                 org-event-helper-descriptor org-event-strategy-root
                 org-event-strategy-initial org-event-strategy-line-forms
                 org-event-strategy-finish-forms org-event-strategy-helpers))
(export run-org-event-strategy)

(def (run-org-event-strategy strategy source (overrides '()))
  (run-event-fold source
                  (org-event-strategy-root strategy)
                  (org-event-strategy-initial strategy)
                  (org-event-strategy-line-forms strategy)
                  (org-event-strategy-finish-forms strategy)
                  (map org-event-helper-descriptor
                       (org-event-strategy-helpers strategy))
                  overrides))
