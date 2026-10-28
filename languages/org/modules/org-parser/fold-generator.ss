;;; -*- Gerbil -*-
;;; Org owns strategy declarations and generated text; Gerbil owns products.
(import (only-in :gerbil-parser/language-build-support
                 event-fold-scheme-module-sources)
        (only-in :gerbil-parser/language-build-support run-build-tool)
        (only-in "fold-package.ss" write-fold-package)
        (only-in :gerbil-parser/language-build-support
                 compile-event-fold-scheme-units)
        (only-in "grammar.ss" org-mode-language-grammar)
        (only-in "event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms org-event-helpers)
        (only-in "values.ss" org-event-helper-descriptor))
(export main)

(def (main output (profile "static"))
  (unless (member profile '("static" "dynamic"))
    (error "unknown Org fold build profile" profile))
  (create-directory* output)
  (let* ((helpers (map org-event-helper-descriptor org-event-helpers))
         (parameters '((configured_inlinetask_min_level inlinetask-min-level 15)
                       (configured_inline_script_policy inline-script-policy 2)))
         (document
          (event-fold-scheme-module-sources
           'parse-org-compiled-events org-mode-language-grammar 'OrgFile
           org-event-initial org-event-line-forms org-event-finish-forms helpers
           parameters))
         (tape
          (event-fold-scheme-module-sources
           'parse-org-compiled-tape org-mode-language-grammar 'OrgFile
           org-event-initial org-event-line-forms org-event-finish-forms
           helpers parameters 64 'tape))
         (inline
          (event-fold-scheme-module-sources
           'parse-org-compiled-inline-events org-mode-language-grammar 'OrgFile '()
           '((call-source-helper inline-span start end ((uint 2)))) '() helpers))
         (units (append document inline tape)))
    (write-fold-package output units)
    (if (equal? profile "dynamic")
      (run-build-tool (list "env" (string-append "GERBIL_PATH=" output)
                         "gxi" (path-expand "build.ss" output) "compile" "--optimized"))
      (compile-event-fold-scheme-units output (map car units)))
    (displayln "GENERATION-OK native-org-document-and-inline")
    (force-output)))
