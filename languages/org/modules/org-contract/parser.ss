;;; -*- Gerbil -*-
;;; Native reference parser for the same Org contract AOT grammar.

(import (only-in :gerbil-parser/language-support deflanguage-parser-loader)
        (only-in "grammar.ss" org-contract-language-grammar))
(export org-contract-language parse-org-contract-expression)

(deflanguage-parser-loader org-contract-language
  (grammar org-contract-language-grammar)
  (parse parse-org-contract-expression))
