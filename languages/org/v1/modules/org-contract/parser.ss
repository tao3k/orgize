;;; -*- Gerbil -*-
;;; Native reference parser for the same Org contract AOT grammar.

(import (only-in :gerbil-parser/src/language/entry deflanguage-parser)
        (only-in "grammar.ss" org-contract-language-grammar))
(export org-contract-language parse-org-contract-expression)

(deflanguage-parser org-contract-language
  (grammar org-contract-language-grammar)
  (parse parse-org-contract-expression))
