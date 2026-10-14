;;; -*- Gerbil -*-
;;; One expectation per block; comments and whitespace are grammar extras.
(import (only-in :gerbil-parser/language-support/grammar deflanguage)
        (only-in :gerbil-parser/src/language/descriptor language-grammar-with-identity))
(export org-expectation-language-grammar org-expectation-grammar
        org-expectation-parser-ir org-expectation-parser)

(deflanguage org-expectation
  (syntax
   (lexical
  (root source-file)
  (lex
   (whitespace Whitespace (whitespace+))
   (comment Comment (line-comment "#"))
   (exists ExistsToken (literals "exists"))
   (not NotToken (literals "not"))
   (count CountToken (literals "count"))
   (operator CompareToken (literals "<=" "<" ">=" ">" "==" "!="))
   (integer CountValueToken (decimal-digit+))
   ;; Reserve unknown words in the lexical catalog; the parser admits no form.
   (invalid InvalidToken (until-delimiters " \t\r\n#<>=!")))
  (extras whitespace comment)
  (keywords)
  (recoveries)
  (conflicts reject)
  (case-insensitive #f)))
  (rules
   (source-file (node ExpectSource (field expectation expectation)))
   (expectation
    (choice (node ExpectExists exists)
            (node ExpectNotExists (seq not exists))
            (node ExpectCount (seq count operator integer))))))

(def org-expectation-language-grammar
  (language-grammar-with-identity org-expectation-syntax "org-expectation" "v1" "org-expectation.v1"))
