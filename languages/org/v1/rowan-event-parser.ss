;;; -*- Gerbil -*-
;;; One Org-owned strategy executes in Scheme and AOT-lowers to Rust/Rowan.

(import (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "modules/org-parser/event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms
                 org-event-helpers)
        (only-in "modules/org-parser/objects.ss" make-org-event-strategy)
        (only-in "modules/org-parser/funs.ss"
                 run-org-event-strategy org-event-strategy-ir-json))
(export parse-org-rowan-events parse-org-rowan-events-with-inlinetask-level
        parse-org-rowan-events-with-inline-script-policy
        parse_org_rowan_events)

(def org-v1-event-strategy
  (make-org-event-strategy 'OrgFile org-event-initial org-event-line-forms
                           org-event-finish-forms org-event-helpers
                           '((configured_inlinetask_min_level inlinetask-min-level 15)
                             (configured_inline_script_policy inline-script-policy 2))))

(def (parse-org-rowan-events source)
  (run-org-event-strategy org-v1-event-strategy source))

(def (parse-org-rowan-events-with-inlinetask-level source level)
  (run-org-event-strategy org-v1-event-strategy source
                          (list (cons 'inlinetask-min-level (max 1 level)))))

(def (parse-org-rowan-events-with-inline-script-policy source policy)
  (unless (memv policy '(0 1 2))
    (error "invalid Org inline script policy" policy))
  (run-org-event-strategy org-v1-event-strategy source
                          (list (cons 'inline-script-policy policy))))

(def parse_org_rowan_events
  (org-event-strategy-ir-json org-v1-event-strategy
                              'parse_org_rowan_events
                              org-v1-language-grammar))
