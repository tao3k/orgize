;;; -*- Gerbil -*-
;;; Org owns strategy declarations and generated text; Gerbil owns products.
(import (only-in :gerbil-parser/event-fold-build-support
                 event-fold-scheme-module-sources make-event-fold-consumer)
        (only-in "fold-package.ss" write-fold-package fold-package-source-directory)
        (only-in :gerbil-parser/event-fold-build-support
                 compile-event-fold-scheme-units)
        (only-in :gerbil-parser/graph-projection-support
                 fold-graph-catalog graph-projection-nodes graph-node-syntax-kind)
        (only-in "../org-elements/graph.ss" org-graph-projection)
        (only-in "grammar.ss" org-mode-language-grammar)
        (only-in "event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms org-event-helpers)
        (only-in "values.ss" org-event-helper-descriptor))
(export main)

(def (graph-configuration-source)
  (let ((rules (fold-graph-catalog org-mode-language-grammar org-graph-projection))
        (names (list->vector (map graph-node-syntax-kind
                                  (graph-projection-nodes org-graph-projection)))))
    (call-with-output-string
     (lambda (port)
       (for-each
        (lambda (form) (write form port) (newline port))
        `((import (only-in :gerbil-parser/event-fold-program-support
                            make-fold-graph fold-graph-start! fold-graph-token!
                            fold-graph-finish! fold-graph-publish!))
          (export make-org-fold-graph fold-graph-start! fold-graph-token!
                  fold-graph-finish! fold-graph-publish!)
          (def rules ',rules)
          (def names ',names)
          ;; Ledger reads file directives, the leading heading and its properties.
          (def (ledger-element? kind parent start)
            (let ((kind (vector-ref names kind))
                  (parent (and parent (vector-ref names parent))))
              (case kind
                ((OrgFile OrgKeyword) #t)
                ((OrgSection) (and (eq? parent 'OrgFile) (= start 0)))
                ((OrgPropertyDrawer) (eq? parent 'OrgSection))
                ((OrgNodeProperty) (eq? parent 'OrgPropertyDrawer))
                (else #f))))
          (def (make-org-fold-graph source digest overrides)
            (let (ledger? (assq 'ledger-source-spans overrides))
              (let (select? (and ledger? (> (cdr ledger?) 0) ledger-element?))
                (make-fold-graph source digest rules (not select?) select?))))))))))

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
         (product
          (event-fold-scheme-module-sources
           'parse-org-compiled-product org-mode-language-grammar 'OrgFile
           org-event-initial org-event-line-forms org-event-finish-forms
           helpers parameters 64
           (make-event-fold-consumer ':orgize-fold/graph-configuration
                                     'make-org-fold-graph 'fold-graph-start!
                                     'fold-graph-token! 'fold-graph-finish! 'fold-graph-publish!)))
         (inline
          (event-fold-scheme-module-sources
           'parse-org-compiled-inline-events org-mode-language-grammar 'OrgFile '()
           '((call-source-helper inline-span start end ((uint 2) (uint 0)))) '() helpers))
         (units (cons (cons "graph-configuration" (graph-configuration-source))
                      (append document inline product)))
         (source-directory (if (equal? profile "dynamic")
                             (fold-package-source-directory output) output)))
    (write-fold-package source-directory units)
    ;; The dynamic package is a registered source dependency of the root
    ;; Module graph. Only that graph compiles it; standalone static controls
    ;; retain their existing native compilation entry.
    (when (equal? profile "static")
      (compile-event-fold-scheme-units output (map car units)))
    (displayln "GENERATION-OK native-org-document-and-inline")
    (force-output)))
