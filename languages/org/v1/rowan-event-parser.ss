;;; -*- Gerbil -*-
;;; One Org-owned strategy executes in Scheme and AOT-lowers to Rust/Rowan.

(import (only-in :gerbil-parser/rust-rowan-event-support
                 run-event-fold event-fold-ir-json)
        (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "modules/org-parser/event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms
                 org-event-helpers))
(export parse-org-rowan-events parse_org_rowan_events)

(def (parse-org-rowan-events source)
  (run-event-fold source 'OrgFile org-event-initial
                  org-event-line-forms org-event-finish-forms
                  org-event-helpers))

(def parse_org_rowan_events
  (event-fold-ir-json 'parse_org_rowan_events org-v1-language-grammar
                      'OrgFile org-event-initial
                      org-event-line-forms org-event-finish-forms
                      org-event-helpers))
