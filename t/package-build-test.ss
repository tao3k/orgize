;;; Native package build ownership; no parser evaluation or publication.
(import :std/test
        (only-in :std/io/tempfile call-with-temporary-directory)
        (only-in :std/error Error?)
        (only-in :std/misc/process run-process)
        (only-in :std/misc/ports read-all-as-string)
        (only-in :std/source this-source-file)
        (only-in :gerbil/expander current-expander-context make-top-context eval-syntax)
        (only-in :gerbil-parser/src/build-support/compiler-profile call-with-compiler-import-progress))
(export package-build-test)
;; Keep expensive grammar construction inside a Case, rather than gxtest's
;; quiet discovery window. Report actual SDK imports, never timer heartbeats.
(def generator-path (path-expand "../languages/org/generate-fold.ss" (path-directory (this-source-file))))
(def generator
  (delay
    (call-with-compiler-import-progress
      (lambda ()
        (parameterize ((current-expander-context (make-top-context)))
          (eval-syntax '(import :gerbil/core))
          (eval-syntax `(import (only-in ,generator-path main)))
          (eval-syntax 'main))))))
(def (fold-main . arguments) (apply (force generator) arguments))
(def (write-fixture root name)
  (def path (path-expand name root))
  (create-directory* (path-directory path))
  (call-with-output-file path (lambda (port) (display "owned fixture\n" port)))
  path)
(def package-build-test
  (test-suite "Native generated fold build ownership"
    (test-case "package and syntax specs import as modules without activating generators"
      (for-each
        (lambda (relative)
          (let* ((entry (path-expand relative (path-directory (this-source-file))))
                 (code
                  `(begin
                     (import :gerbil/expander)
                     (let ((evaluate (gx#current-expander-module-eval)))
                       (parameterize
                        ((gx#current-expander-module-eval
                          (lambda (context)
                            (when (and (gx#module-context? context)
                                       (memq (gx#expander-context-id context)
                                        '(orgize/languages/org/grammar orgize/languages/org/generate-fold)))
                              (error "metadata request activated a generator"))
                            (evaluate context))))
                        (parameterize ((gx#current-expander-context (gx#make-top-context)))
                          (gx#eval-syntax '(import :gerbil/core))
                          (gx#eval-syntax '(import (only-in ,entry main)))
                          (gx#eval-syntax '(main "spec")))))))
                 (output (run-process (list "gxi" "-:max-heap=1G,debug=q" "-e" (object->string code))
                           stdout-redirection: #t coprocess: read-all-as-string)))
            (check-equal? (list? (call-with-input-string output read)) #t)))
        '("../build.ss" "../build-parser.ss")))
    (test-case "clean removes every owned generation and preserves adjacent products and sources"
      (call-with-temporary-directory "org-fold-clean-test"
        (lambda (root)
          (def owned
            (map (lambda (name) (write-fixture root name))
              '("lib/orgize/fold/parse-org-compiled-events.ssi"
                "lib/orgize/fold/parse-org-compiled-events.o1"
                "lib/orgize/fold/parse-org-compiled-events.o2"
                "lib/orgize/fold/parse-org-compiled-events~0.o1"
                "lib/static/orgize__fold__parse-org-compiled-events.scm")))
          (def neighbor (write-fixture root "lib/orgize/fold/neighbor.o1"))
          (def source (write-fixture root "parse-org-compiled-events.ss"))
          (fold-main root "dynamic" "clean")
          (check-equal? (ormap file-exists? owned) #f)
          (check-equal? (file-exists? neighbor) #t)
          (check-equal? (file-exists? source) #t)
          ;; Repeated clean is idempotent; it must not widen ownership.
          (fold-main root "dynamic" "clean")
          (check-equal? (file-exists? neighbor) #t))))
    (test-case "invalid profile or action refuses before changing products"
      (call-with-temporary-directory "org-fold-clean-refusal"
        (lambda (root)
          (def product (write-fixture root "lib/orgize/fold/parse-org-compiled-events.o1"))
          (check-exception (fold-main root "dynamic" "erase") Error?)
          (check-exception (fold-main root "static" "clean") Error?)
          (check-exception (fold-main root "unknown" "clean") Error?)
          (check-equal? (file-exists? product) #t))))))
