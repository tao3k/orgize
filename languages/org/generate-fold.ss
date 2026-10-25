;;; -*- Gerbil -*-
;;; Native build dispatch: cleanup never imports or constructs a grammar.
(import (only-in :std/source this-source-file)
        (only-in :gerbil/expander current-expander-context make-top-context eval-syntax)
        (only-in "./fold-build.ss" validate-fold-action clean-fold-products!))
(export main)
(def generator-path (path-expand "fold-generator.ss" (path-directory (this-source-file))))
(def compiler
  (delay
    (parameterize ((current-expander-context (make-top-context)))
      (eval-syntax '(import :gerbil/core))
      (eval-syntax `(import (only-in ,generator-path main)))
      (eval-syntax 'main))))
(def (main output (profile "static") (action "compile"))
  (validate-fold-action profile action)
  (if (equal? action "clean")
    (clean-fold-products! output)
    ((force compiler) output profile)))
