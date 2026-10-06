;;; -*- Gerbil -*-
(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/v1/modules/org-parser/value-funs.ss" org-value-plan))
(export org-native-source-value-test)
(def org-native-source-value-test
  (test-suite "Native downstream source grammar closure"
    (test-case "Include ranges and modes"
      (check (org-value-plan '("include-lines" "2-9")) => '(("range" "2" "9")))
      (check (org-value-plan '("include-lines" "-3")) => '(("range" "" "3")))
      (check (org-value-plan '("include-lines" "0-0")) => '(("invalid" "" "")))
      (check (org-value-plan '("include-mode" "SRC" "scheme")) => '(("src")))
      (check (org-value-plan '("include-mode")) => '(("org"))))
    (test-case "Babel variables references and inert literals"
      (check (org-value-plan '("source-variable" " x = foo(a=1) ")) => '(("x" "foo(a=1)" "true")))
      (check (org-value-plan '("noweb-references" "<<one>> <<two(x=1)>> <<bad name>>")) => '(("one" "two")))
      (check (org-value-plan '("var-target" "source(x=[1,2])")) => '(("source" "call")))
      (check (org-value-plan '("var-target" "named[1:2]")) => '(("named" "named")))
      (check (org-value-plan '("header-policy" "noweb" "yes strip-tangle")) => '(("strip")))
      (check (org-value-plan '("header-policy" "comments" "NOWEB")) => '(("noweb")))
      (check (org-value-plan '("header-policy" "result" "not-a-result")) => '(("other")))
      (check (org-value-plan '("header-language" "HEADER-ARGS:Scheme" "scheme")) => '(("true")))
      (check (org-value-plan '("header-defaults" "inline")) =>
             '(("eval" "yes") ("session" "none") ("results" "replace") ("exports" "results")
               ("cache" "no") ("noweb" "no") ("hlines" "yes") ("tangle" "no")))
      (for-each (lambda (raw) (check (org-value-plan (list "var-target" raw)) => '()))
                '("" "1e2" "NaN" "(system \"touch forbidden\")" "true" "[1]")))
    (test-case "PLOT quote and balanced token policies"
      (check (org-value-plan '("plot-options" "title:\"two words\" deps:(1 2)")) =>
             '(("title" "two words" "title:\"two words\"") ("deps" "(1 2)" "deps:(1 2)")))
      (check (org-value-plan '("column-list" "(1, 2 0 bad 3)")) => '(("1" "2" "3")))
      (check (org-value-plan '("alignment-cookie" " <r> ")) => '(("true")))
      (check (org-value-plan '("alignment-cookie" " <r ")) => '(("false")))
      (check (org-value-plan '("option-kind" "DEPS" "plot")) => '(("dep"))))
    (test-case "Radio optional translator and receiver marker"
      (check (org-value-plan '("radio-header" "SEND target")) => '(("target" "" "false")))
      (check (org-value-plan '("radio-options" "SEND target")) => '())
      (check (org-value-plan '("radio-options" "SEND target orgtbl-to-tsv :skip 2 :splice")) =>
             '(("skip" "2" "true" ":skip 2") ("splice" "" "false" ":splice")))
      (check (org-value-plan '("radio-marker" "<!-- BEGIN RECEIVE ORGTBL target -->" "begin receive orgtbl")) => '(("target"))))
    (test-case "Contract links macros and stable field presence"
      (check (org-value-plan '("contract-reference" "[[file:./contracts.org#id][Title]]")) =>
             '(("[[file:./contracts.org#id][Title]]" "contracts.org" "true" "id" "true")))
      (check (org-value-plan '("contract-reference" "{{{contract(id)}}}")) =>
             '(("{{{contract(id)}}}" "" "false" "id" "true")))
      (check (org-value-plan '("contract-values" "[[file:a.org#x][two words]], {{{contract(y)}}}")) =>
             '(("[[file:a.org#x][two words]]" "{{{contract(y)}}}")))
      (check (org-value-plan '("severity" " WARN ")) => '(("warning")))
      (check (org-value-plan '("contract-policy" "kind" " org-elements ")) => '(("org-elements")))
      (check (org-value-plan '("contract-policy" "kind" "ORG-ELEMENTS")) => '(("other")))
      (check (org-value-plan '("contract-policy" "scope" " SUBTREE ")) => '(("subtree")))
      (check (org-value-plan '("contract-policy" "language" " Org-Elements-Query-Expr ")) => '(("query")))
      (check (org-value-plan '("contract-policy" "named-id" " id.MESSAGE ")) => '(("id.MESSAGE")))
      (check (org-value-plan '("contract-policy" "named-id" " id.message ")) => '(("")))
      (check (org-value-plan '("contract-policy" "named-id" " id.fix ")) => '(("")))
      (check (org-value-plan '("contract-qualified-link" "[[file:a.org#id]]" "a.org" "id")) => '(("true")))
      (check (org-value-plan '("contract-qualified-link" " [[file:a.org#id]] " "a.org" "id")) => '(("false")))
      (check (org-value-plan '("contract-qualified-link" "[[file:a.org]]" "a.org" "")) => '(("false"))))))
