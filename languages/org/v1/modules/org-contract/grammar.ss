;;; -*- Gerbil -*-
;;; Org-owned contract expression syntax; gerbil-parser owns the AOT engine.

(import (only-in :gerbil-parser/language-support deflanguage))
(export org-contract-language-grammar
        org-contract-grammar
        org-contract-parser-ir
        org-contract-parser)

(deflanguage org-contract
  (identity "org-contract" "v1" "org-contract-expression.v1")
  (root source-file)
  (lex
   (whitespace Whitespace (whitespace+))
   (comment Comment (line-comment ";"))
   (open-paren OpenParen (literals "("))
   (close-paren CloseParen (literals ")"))
   (string ContractStringToken (escaped-quoted-string "\""))
   (atom ContractAtomToken (until-delimiters " \t\r\n();\"")))
  (rules
   (source-file
    (node ContractSource
      (repeat (field expression expression))))
   (expression
    (choice list-expression atom-expression string-expression))
   (list-expression
    (node ContractList
      (seq (literal "(")
           (repeat (field item expression))
           (literal ")"))))
   (atom-expression
    (node ContractAtom (field value atom)))
   (string-expression
    (node ContractString (field value string))))
  (extras whitespace comment)
  (keywords)
  (recoveries)
  (conflicts reject)
  (case-insensitive #f))
