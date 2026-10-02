;;; -*- Gerbil -*-
;;; Scheme-owned recognition of lifecycle line shapes inside LOGBOOK drawers.

(import (only-in :std/string/misc string-trim)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure scheme-pure->rust string-after string-before
                 string-prefix? string-trim-start))
(export logbook-content-line logbook-content-line-rust
        logbook-line-kind logbook-line-kind-rust
        logbook-state-quote-shape logbook-state-quote-shape-rust
        logbook-state-to logbook-state-to-rust
        logbook-state-from logbook-state-from-rust
        logbook-clock-duration-shape logbook-clock-duration-shape-rust
        logbook-clock-duration-value logbook-clock-duration-value-rust)

;; The caller supplies one trimmed LOGBOOK line without its optional list dash.
;; Value extraction remains a separate semantic projection over that line.
(define-rust-pure logbook-content-line logbook-content-line-rust
  ((line "&str")) "&str"
  (let* ((trimmed (string-trim line)))
    (if (string-prefix? trimmed "-")
      (string-trim-start (string-after trimmed "-"))
      trimmed)))

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

;; The four delimiter checks preserve empty quoted states while distinguishing
;; them from missing quotes. Rust consumes only these Scheme-authored values.
(define-rust-pure logbook-state-quote-shape logbook-state-quote-shape-rust
  ((line "&str")) "&'static str"
  (let* ((before-first (string-before line "\""))
         (after-first (string-after line "\""))
         (to (string-before after-first "\""))
         (after-second (string-after after-first "\""))
         (before-third (string-before after-second "\""))
         (after-third (string-after after-second "\""))
         (from (string-before after-third "\"")))
    (if (or (equal? before-first line)
            (equal? to after-first)
            (equal? before-third after-second)
            (equal? from after-third))
      "incomplete" "complete")))

(define-rust-pure logbook-state-to logbook-state-to-rust
  ((line "&str")) "&str"
  (let* ((after-first (string-after line "\"")))
    (string-before after-first "\"")))

(define-rust-pure logbook-state-from logbook-state-from-rust
  ((line "&str")) "&str"
  (let* ((after-first (string-after line "\""))
         (after-second (string-after after-first "\""))
         (after-third (string-after after-second "\"")))
    (string-before after-third "\"")))

;; List-item CLOCK lines are paragraphs rather than structural OrgClock nodes.
;; Keep their duration boundary in Scheme while Rust owns the typed duration.
(define-rust-pure logbook-clock-duration-shape logbook-clock-duration-shape-rust
  ((line "&str")) "&'static str"
  (if (equal? (string-before line "=>") line) "absent" "present"))

(define-rust-pure logbook-clock-duration-value logbook-clock-duration-value-rust
  ((line "&str")) "&str"
  (string-trim (string-after line "=>")))
