;;; -*- Gerbil -*-
;;; Org-owned contract expression syntax; gerbil-parser owns the AOT engine.

(import (only-in :gerbil-parser/language-support/grammar deflanguage)
        (only-in :gerbil-parser/src/language/descriptor language-grammar-with-identity))
(export org-contract-language-grammar
        org-contract-grammar
        org-contract-parser-ir
        org-contract-parser)

(deflanguage org-contract
  (syntax
   (lexical
  (root source-file)
  (lex
   (whitespace Whitespace (whitespace+))
   (comment Comment (line-comment ";"))
   (open-paren OpenParen (literals "("))
   (close-paren CloseParen (literals ")"))
   (string ContractStringToken (escaped-quoted-string "\""))
   (atom ContractAtomToken (until-delimiters " \t\r\n();\"")))
  (extras whitespace comment)
  (keywords)
  (recoveries)
  (conflicts reject)
  (case-insensitive #f)))
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
    (node ContractString (field value string)))))

(def org-contract-language-grammar
  (language-grammar-with-identity org-contract-syntax "org-contract" "v1" "org-contract-expression.v1"))
