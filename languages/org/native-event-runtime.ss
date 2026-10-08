;;; -*- Gerbil -*-
;;; Runtime-only projection of the same Org-owned POO strategy.
(import (only-in "modules/org-parser/event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms
                 org-event-helpers)
        (only-in "modules/org-parser/objects.ss" make-org-event-strategy)
        (only-in "modules/org-parser/runtime-funs.ss" run-org-event-strategy))
(export org-event-strategy parse-org-native-events parse-org-inline-events
        parse-org-native-events-with-parameters
        parse-org-native-events-with-inlinetask-level
        parse-org-native-events-with-inline-script-policy)

(def org-event-strategy
  (make-org-event-strategy 'OrgFile org-event-initial org-event-line-forms
                           org-event-finish-forms org-event-helpers
                           '((configured_inlinetask_min_level inlinetask-min-level 15)
                             (configured_inline_script_policy inline-script-policy 2))))

(def (parse-org-native-events source)
  (run-org-event-strategy org-event-strategy source))

;; Property secondary values use the same inline helpers, never file syntax.
(def org-inline-strategy
  (make-org-event-strategy 'OrgFile '()
    '((call-source-helper inline-span start end ((uint 2))))
    '() org-event-helpers))

(def (parse-org-inline-events source)
  (run-org-event-strategy org-inline-strategy source))

(def (parse-org-native-events-with-parameters source level policy)
  (unless (and (exact-integer? level) (> level 0) (memv policy '(0 1 2)))
    (error "invalid Org parser parameters" level policy))
  (run-org-event-strategy org-event-strategy source
                          (list (cons 'inlinetask-min-level level)
                                (cons 'inline-script-policy policy))))

(def (parse-org-native-events-with-inlinetask-level source level)
  (run-org-event-strategy org-event-strategy source
                          (list (cons 'inlinetask-min-level (max 1 level)))))

(def (parse-org-native-events-with-inline-script-policy source policy)
  (unless (memv policy '(0 1 2))
    (error "invalid Org inline script policy" policy))
  (run-org-event-strategy org-event-strategy source
                          (list (cons 'inline-script-policy policy))))
