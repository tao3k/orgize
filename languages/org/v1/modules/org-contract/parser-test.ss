;;; -*- Gerbil -*-
;;; Exact native AST contract for the Org-owned expression grammar.

(import (only-in :std/test check test-case test-suite)
        (only-in :gerbil-parser/src/runtime/artifact
                 parse-artifact-roundtrip parse-artifact-success?)
        (only-in :gerbil-parser/src/testing/parser-ast check-parser-ast)
        (only-in "parser.ss" parse-org-contract-expression))
(export org-contract-parser-test)

(def org-contract-parser-test
  (test-suite "Org contract expression grammar"
    (test-case "atom AST has an exact field, token, and byte span"
      (check-parser-ast (parse-org-contract-expression "count") "count"
        (node ContractSource 0 5
          (field expression 0 5
            (node ContractAtom 0 5
              (field value 0 5
                (token atom "count" 0 5)))))))
    (test-case "Unicode whitespace and adjacent strings roundtrip"
      (let* ((source "(query π \"one\"\"two\") ; note\n")
             (artifact (parse-org-contract-expression source)))
        (check (parse-artifact-success? artifact) => #t)
        (check (parse-artifact-roundtrip artifact) => source)))
    (test-case "malformed list and unmatched delimiter are rejected"
      (check (parse-artifact-success?
              (parse-org-contract-expression "(count")) => #f)
      (check (parse-artifact-success?
              (parse-org-contract-expression ")")) => #f))))
