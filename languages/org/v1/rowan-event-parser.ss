;;; -*- Gerbil -*-
;;; One Org-owned strategy executes in Scheme and AOT-lowers to Rust/Rowan.

(import (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "modules/org-parser/event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms
                 org-event-helpers)
        (only-in "modules/org-parser/objects.ss" make-org-event-strategy)
        (only-in "modules/org-parser/funs.ss"
                 run-org-event-strategy org-event-strategy-ir-json))
(export parse-org-rowan-events parse_org_rowan_events)

(def org-v1-event-strategy
  (make-org-event-strategy 'OrgFile org-event-initial org-event-line-forms
                           org-event-finish-forms org-event-helpers))

(def (parse-org-rowan-events source)
  (run-org-event-strategy org-v1-event-strategy source))

(def parse_org_rowan_events
  (org-event-strategy-ir-json org-v1-event-strategy
                              'parse_org_rowan_events
                              org-v1-language-grammar))
