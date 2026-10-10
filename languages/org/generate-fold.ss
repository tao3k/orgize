;;; -*- Gerbil -*-
;;; Build-only Org declarations; the engine owns all native lowering.
(import (only-in :gerbil-parser/src/compiler/event-fold-scheme-modules
                 event-fold-scheme-module-sources)
        (only-in :std/make make)
        (only-in :std/misc/ports read-all-as-string)
        (only-in :gerbil/runtime/loader __find-library-module)
        (only-in :gerbil-parser/src/compiler/event-fold-scheme-build
                 compile-event-fold-scheme-units)
        (only-in "./grammar.ss" org-mode-language-grammar)
        (only-in "./modules/org-parser/event-strategy.ss"
                 org-event-initial org-event-line-forms org-event-finish-forms org-event-helpers)
        (only-in "./modules/org-parser/objects.ss" org-event-helper-descriptor))
(export main)

(def (write-generated-source path source)
  (unless (and (file-exists? path)
               (equal? source (call-with-input-file path read-all-as-string)))
    (call-with-output-file path (lambda (port) (display source port)))))

(def (runtime-products output name)
  (map (lambda (suffix)
         (path-expand (string-append "lib/orgize/fold/" name suffix) output))
       '("" "~0")))

(def (runtime-product-fresh? source module-path)
  ;; Use the same versioned object selection as the SDK loader. A source-only
  ;; fallback is incomplete for this profile, even when SSI/static inputs exist.
  (let (product (__find-library-module module-path))
    (and product (not (equal? (path-extension product) ".scm"))
         (>= (time->seconds
               (file-info-last-modification-time (file-info product)))
             (time->seconds
               (file-info-last-modification-time (file-info source)))))))

(def (require-runtime-products! output names)
  (for-each
    (lambda (name)
      (let (source (path-expand (string-append name ".ss") output))
        (for-each
          (lambda (product)
            (unless (runtime-product-fresh? source product)
              (error "incomplete Org fold runtime product" product)))
          (runtime-products output name))))
    names))

(def (main output (profile "static"))
  (unless (member profile '("static" "dynamic"))
    (error "unknown Org fold build profile" profile))
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
    (write-generated-source (path-expand "gerbil.pkg" output)
                            "(package: orgize/fold)\n")
    (for-each
     (lambda (unit)
       (write-generated-source
         (path-expand (string-append (car unit) ".ss") output) (cdr unit)))
     units)
    (if (equal? profile "dynamic")
      ;; Package loading needs dynamic products in addition to native inputs.
      ;; The producer owns both profiles; consumers do not generate parsers.
      (begin
        ;; std/make's SDK output set includes SSI/static inputs. Invalidate the
        ;; owned static marker when an interrupted job left dynamic products
        ;; absent or older than their source, then let std/make rebuild them.
        (for-each
          (lambda (unit)
            (let* ((name (car unit))
                   (source (path-expand (string-append name ".ss") output))
                   (marker (path-expand
                             (string-append "lib/static/orgize__fold__" name ".scm")
                             output)))
              (unless (andmap (lambda (product)
                               (runtime-product-fresh? source product))
                             (runtime-products output name))
                (when (file-exists? marker) (delete-file marker)))))
          units)
        (make (map car units)
              srcdir: output prefix: "orgize/fold"
              libdir: (path-expand "lib" output) optimize: #t)
        (require-runtime-products! output (map car units)))
      (compile-event-fold-scheme-units output (map car units)))
    (displayln "GENERATION-OK native-org-document-and-inline")
    (force-output)))
