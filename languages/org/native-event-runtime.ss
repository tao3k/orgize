;;; -*- Gerbil -*-
;;; Native generated Scheme products only; declarations stay in the build lane.
(import (only-in :orgize/native-fold/parse-org-compiled-events parse-org-compiled-events)
        (only-in :orgize/native-fold/parse-org-compiled-inline-events parse-org-compiled-inline-events))
(export parse-org-native-events parse-org-inline-events
        parse-org-native-events-with-parameters
        parse-org-native-events-with-inlinetask-level
        parse-org-native-events-with-inline-script-policy)

(def (parse-org-native-events source)
  (parse-org-compiled-events source))

;; Property secondary values use the same inline helpers, never file syntax.
(def (parse-org-inline-events source)
  (parse-org-compiled-inline-events source))

(def (parse-org-native-events-with-parameters source level policy)
  (unless (and (exact-integer? level) (> level 0) (memv policy '(0 1 2)))
    (error "invalid Org parser parameters" level policy))
  (parse-org-compiled-events source
                          (list (cons 'inlinetask-min-level level)
                                (cons 'inline-script-policy policy))))

(def (parse-org-native-events-with-inlinetask-level source level)
  (parse-org-compiled-events source
                          (list (cons 'inlinetask-min-level (max 1 level)))))

(def (parse-org-native-events-with-inline-script-policy source policy)
  (unless (memv policy '(0 1 2))
    (error "invalid Org inline script policy" policy))
  (parse-org-compiled-events source
                          (list (cons 'inline-script-policy policy))))
