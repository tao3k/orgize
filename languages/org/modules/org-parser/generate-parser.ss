#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only AOT entry. Cargo consumes the committed Rust product.

(import (only-in "catalog-generator.ss" generate-language-catalog)
        (only-in "grammar.ss" org-mode-language-grammar))

(export main)

(def (main output-path)
  (generate-language-catalog output-path org-mode-language-grammar))
