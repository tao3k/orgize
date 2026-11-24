#!/usr/bin/env gxi
;;; -*- Gerbil -*-
(import (only-in "../org-parser/catalog-generator.ss" generate-language-catalog)
        (only-in "grammar.ss" org-contract-language-grammar))
(export main)

(def (main output-path)
  (generate-language-catalog output-path org-contract-language-grammar))
