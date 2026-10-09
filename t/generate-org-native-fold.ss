;;; -*- Gerbil -*-
;;; Thin consumer: the shared engine owns admission and native code generation.
(import (only-in :std/misc/ports read-all-as-string)
        (only-in :gerbil-parser/src/compiler/event-fold-scheme-modules event-fold-scheme-module-sources)
        (only-in :gerbil-parser/src/compiler/event-fold-runtime run-event-fold)
        (only-in :orgize/languages/org/grammar org-mode-language-grammar)
        (only-in :orgize/languages/org/modules/org-parser/event-strategy
                 org-event-initial org-event-line-forms org-event-finish-forms org-event-helpers)
        (only-in :orgize/languages/org/modules/org-parser/objects org-event-helper-descriptor))
(export main)

(def (main output controls)
  (let* ((helpers (map org-event-helper-descriptor org-event-helpers))
         (parameters '((configured_inlinetask_min_level inlinetask-min-level 15)
                       (configured_inline_script_policy inline-script-policy 2)))
         (generated (event-fold-scheme-module-sources
                     'parse-org-compiled-events org-mode-language-grammar 'OrgFile
                     org-event-initial org-event-line-forms org-event-finish-forms
                     helpers parameters))
         (corpus
          (append
           (map (lambda (path) (call-with-input-file path read-all-as-string))
                '("languages/org/fixtures/native-event-source.org" "benches/fixtures/doc.org"))
           '("" "é\r\nλ\rtail" "* TODO Heading :tag:\nSCHEDULED: <2026-10-09 Fri>\n"
             "* H\n:PROPERTIES:\n:ID: x\n:END:\nbody *bold* [[url][text]]\n"
             "#+begin_src scheme\n(display \"x\")\n#+end_src\n"
             "- item\n  1. child\n| a | b |\n#+TBLFM: $1=$2\n"))))
    (for-each
     (lambda (unit)
       (call-with-output-file
        (path-expand (string-append (car unit) ".ss") (path-directory output))
        (lambda (port) (display (cdr unit) port))))
     generated)
    (call-with-output-file (path-expand "gerbil.pkg" (path-directory output))
      (lambda (port) (write '(package: orgize-native-fold) port) (newline port)))
    (call-with-output-file (path-expand "build-fold.ss" (path-directory output))
      (lambda (port)
        (write '(import (only-in :gerbil-parser/src/compiler/event-fold-scheme-build
                                compile-event-fold-scheme-units)) port) (newline port)
        (write '(export main) port) (newline port)
        (write `(def (main) (compile-event-fold-scheme-units
                            ,(path-directory output) ',(map car generated))) port)
        (newline port)))
    (call-with-output-file output
      (lambda (port)
        (write '(import "./parse-org-compiled-events.ss") port)
        (display "\n(export main)\n" port)
        (write
         '(def (main controls)
            (for-each
             (lambda (control)
               (unless (equal? (parse-org-compiled-events (car control) (cadr control))
                               (caddr control))
                 (error "complete native Org strategy mismatch"))
               (displayln "CASE-OK native-org")
               (force-output))
             (call-with-input-file controls read))
            (displayln "OK") (force-output))
         port)
        (newline port)))
    (call-with-output-file controls
      (lambda (port)
        (write
         (apply append
          (map (lambda (source)
                 (map (lambda (overrides)
                        (list source overrides
                              (run-event-fold source 'OrgFile org-event-initial
                                              org-event-line-forms org-event-finish-forms
                                              helpers overrides)))
                      '(() ((inlinetask-min-level . 2) (inline-script-policy . 0))
                        ((inlinetask-min-level . 15) (inline-script-policy . 2)))))
               corpus))
         port)))
    (displayln "GENERATION-OK complete-org-strategy")
    (force-output)))
