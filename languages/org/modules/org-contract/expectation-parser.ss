;;; -*- Gerbil -*-
(import (only-in :gerbil-parser/language-support deflanguage-parser-loader)
        (only-in "expectation-grammar.ss" org-expectation-language-grammar))
(export parse-org-expectation org-expectation-language)
(deflanguage-parser-loader org-expectation-language
  (grammar org-expectation-language-grammar)
  (parse parse-org-expectation))
