;;; -*- Gerbil -*-
;;; Scheme-owned recognition of lifecycle line shapes inside LOGBOOK drawers.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 scheme-pure->rust string-prefix?))
(export logbook-line-kind logbook-line-kind-rust)

;; The caller supplies one trimmed LOGBOOK line without its optional list dash.
;; Value extraction remains a separate semantic projection over that line.
(def logbook-line-rules
  '(("state" "State ")
    ("note" "Note taken on")
    ("refile" "Refiled" "Refiling")
    ("reschedule" "Rescheduled")
    ("redeadline" "New deadline" "Deadline" "Removed deadline")
    ("clock" "CLOCK:")))

(def (logbook-line-kind line)
  (or (ormap (lambda (rule)
               (and (ormap (lambda (prefix) (string-prefix? line prefix))
                           (cdr rule))
                    (car rule)))
             logbook-line-rules)
      "note"))

;; Build the bounded AOT expression from the same rule table at generation
;; time. Its Rust lowering remains a short-circuit branch chain with no
;; per-line list construction or FFI boundary.
(def (logbook-prefix-form prefixes)
  (let (predicates
        (map (lambda (prefix) `(string-prefix? line ,prefix)) prefixes))
    (if (null? (cdr predicates)) (car predicates)
        (cons 'or predicates))))

(def logbook-line-kind-rust
  (scheme-pure->rust
   'logbook-line-kind '((line . "&str")) "&str"
   (foldr (lambda (rule otherwise)
            `(if ,(logbook-prefix-form (cdr rule))
                 ,(car rule) ,otherwise))
          "note" logbook-line-rules)))
