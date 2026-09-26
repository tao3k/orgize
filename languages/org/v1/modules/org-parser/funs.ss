;;; -*- Gerbil -*-
;;; Both Scheme execution and Rust AOT consume one admitted POO strategy.

(import (only-in :gerbil-parser/rust-rowan-event-support
                 run-event-fold event-fold-ir-json)
        (only-in "objects.ss"
                 org-event-helper-descriptor
                 org-event-strategy-root org-event-strategy-initial
                 org-event-strategy-line-forms
                 org-event-strategy-finish-forms
                 org-event-strategy-helpers))
(export run-org-event-strategy org-event-strategy-ir-json)

(def (run-org-event-strategy strategy source)
  (run-event-fold source
                  (org-event-strategy-root strategy)
                  (org-event-strategy-initial strategy)
                  (org-event-strategy-line-forms strategy)
                  (org-event-strategy-finish-forms strategy)
                  (map org-event-helper-descriptor
                       (org-event-strategy-helpers strategy))))

(def (org-event-strategy-ir-json strategy function-name grammar)
  (event-fold-ir-json function-name grammar
                      (org-event-strategy-root strategy)
                      (org-event-strategy-initial strategy)
                      (org-event-strategy-line-forms strategy)
                      (org-event-strategy-finish-forms strategy)
                      (map org-event-helper-descriptor
                           (org-event-strategy-helpers strategy))))
