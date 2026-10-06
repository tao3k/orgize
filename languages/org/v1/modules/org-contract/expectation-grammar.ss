;;; -*- Gerbil -*-
;;; One expectation per block; comments and whitespace are grammar extras.
(import (only-in :gerbil-parser/language-support deflanguage))
(export org-expectation-language-grammar org-expectation-grammar
        org-expectation-parser-ir org-expectation-parser)

(deflanguage org-expectation
  (identity "org-expectation" "v1" "org-expectation.v1")
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
  (rules
   (source-file (node ExpectSource (field expectation expectation)))
   (expectation
    (choice (node ExpectExists exists)
            (node ExpectNotExists (seq not exists))
            (node ExpectCount (seq count operator integer)))))
  (extras whitespace comment)
  (keywords)
  (recoveries)
  (conflicts reject)
  (case-insensitive #f))
