;;; Org-specific native dependency boundaries; no Core lifecycle reimplementation.
(import (only-in :std/test test-suite test-case check-equal?)
        (only-in :std/source this-source-file)
        (only-in :std/misc/process run-process)
        (only-in :std/misc/ports read-all-as-string))
(export org-module-boundary-test)
(def init-path (path-expand "../init.ss" (path-directory (this-source-file))))
(def (run-expression code)
  (run-process (list "gxi" "-:max-heap=1G,debug=q" "-e" (object->string code))
    stdout-redirection: #t coprocess: read-all-as-string))
(def org-module-boundary-test
  (test-suite "Org native dependency boundaries"
    (test-case "business value APIs do not load Module authoring roles"
      (def output
        (run-expression
          `(begin
             (import ,(path-expand "../languages/org/modules/org-parser/values.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-elements/value-types.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-elements/values.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-elements/query.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-elements/syntax.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-contract/value-types.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-contract/values.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-contract/evaluation.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-contract/config.ss"
                        (path-directory (this-source-file)))
                     ,(path-expand "../languages/org/modules/org-contract/syntax.ss"
                        (path-directory (this-source-file))))
             (when (ormap
                     (lambda (owner)
                       (or (assoc (string-append "orgize/languages/org/modules/" owner "/types~0")
                                  (list-modules))
                           (assoc (string-append "orgize/languages/org/modules/" owner "/objects~0")
                                  (list-modules))))
                     '("org-parser" "org-elements" "org-contract"))
               (error "business values activated Module authoring roles"))
             (when (ormap
                     (lambda (owner)
                       (assoc (string-append "orgize/languages/org/modules/"
                                             owner "/runtime-interface~0")
                              (list-modules)))
                     '("org-elements" "org-contract"))
               (error "business values loaded a retired runtime umbrella"))
             (displayln "BUSINESS-VALUES-ISOLATED-OK"))))
      (check-equal? (and (string-contains output "BUSINESS-VALUES-ISOLATED-OK") #t) #t))
    (test-case "native consumers load only their required generated programs"
      (for-each
        (lambda (consumer)
          (def source
            (path-expand (car consumer)
              (path-directory init-path)))
          (def expected (cadr consumer))
          (def forbidden (caddr consumer))
          (def output
            (run-expression
              `(begin
                 (import ,source)
                 (def loaded (map car (list-modules)))
                 (unless (member ,expected loaded)
                   (error "consumer did not load its native program" ,source loaded))
                 (when (ormap (lambda (name) (member name loaded)) ',forbidden)
                   (error "consumer loaded another native program" ,source loaded))
                 (displayln "CONSUMER-ISOLATED-OK"))))
          (check-equal? (and (string-contains output "CONSUMER-ISOLATED-OK") #t) #t))
        '(("languages/org/modules/org-parser/macro-funs.ss"
           "orgize-fold/parse-org-compiled-inline-events~0"
           ("orgize-fold/parse-org-compiled-events~0" "orgize-fold/parse-org-compiled-tape~0"))
          ("languages/org/modules/org-parser/event-runtime.ss"
           "orgize-fold/parse-org-compiled-events~0"
           ("orgize-fold/parse-org-compiled-inline-events~0" "orgize-fold/parse-org-compiled-tape~0"))
          ("bindings/c/event-tape.ss"
           "orgize-fold/parse-org-compiled-tape~0"
           ("orgize-fold/parse-org-compiled-events~0")))))
))
