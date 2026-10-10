;;; -*- Gerbil -*-
;;; Build-only Org declarations; the engine owns all native lowering.
(import (only-in :gerbil-parser/src/compiler/event-fold-scheme-modules
                 event-fold-scheme-module-sources)
        (only-in :gerbil-parser/src/compiler/event-fold-scheme-build
                 compile-event-fold-scheme-units)
        (only-in "./grammar.ss" org-mode-language-grammar)
        (only-in "./modules/org-parser/event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms org-event-helpers)
        (only-in "./modules/org-parser/objects.ss" org-event-helper-descriptor))
(export main)

(def (main output)
  (let* ((helpers (map org-event-helper-descriptor org-event-helpers))
         (document
          (event-fold-scheme-module-sources
           'parse-org-compiled-events org-mode-language-grammar 'OrgFile
           org-event-initial org-event-line-forms org-event-finish-forms helpers
           '((configured_inlinetask_min_level inlinetask-min-level 15)
             (configured_inline_script_policy inline-script-policy 2))))
         (inline
          (event-fold-scheme-module-sources
           'parse-org-compiled-inline-events org-mode-language-grammar 'OrgFile '()
           '((call-source-helper inline-span start end ((uint 2)))) '() helpers))
         (units (append document inline)))
    (call-with-output-file (path-expand "gerbil.pkg" output)
      (lambda (port) (write '(package: orgize/fold) port) (newline port)))
    (for-each
     (lambda (unit)
       (call-with-output-file (path-expand (string-append (car unit) ".ss") output)
         (lambda (port) (display (cdr unit) port))))
     units)
    (compile-event-fold-scheme-units output (map car units))
    (displayln "GENERATION-OK native-org-document-and-inline")
    (force-output)))
