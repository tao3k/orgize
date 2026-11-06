;;; -*- Gerbil -*-
(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/modules/org-parser/value-funs.ss" org-value-plan))
(export org-source-value-test)
(def org-source-value-test
  (test-suite "Native downstream source grammar closure"
    (test-case "Include ranges and modes"
      ;; Batch projection must match the existing scalar native algorithms.
      (let* ((inputs '(("COLOR_ALL+" "'two words' red")
                       ("COLOR+" "blue") ("_ALL" "")
                       ("Mixed_all" "\"\" ünicode") ("é_ALL" "one\\ two")))
             (rows (org-value-plan
                    (cons "property-profile-plan" (apply append inputs)))))
        (check (length rows) => (length inputs))
        (for-each
         (lambda (input row)
           (let* ((key (car input)) (value (cadr input))
                  (name-rows (org-value-plan (list "descriptor-name" key)))
                  (name (and (pair? name-rows) (caar name-rows))))
             (check row =>
                    (append (car (org-value-plan (list "descriptor-key" key)))
                            (list (if name "true" "false") (or name ""))
                            (car (org-value-plan (list "property-tokens" value)))))))
         inputs rows))
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
      (check (org-value-plan '("contract-qualified-link" "[[file:a.org]]" "a.org" "")) => '(("false"))))
    (test-case "Contract block batch matches scalar native policy"
      (let* ((blocks '((" Org-Elements-Query-Expr " "  first " ":name message")
                       ("org-contract" "id.message" ":severity WARN :name fix")
                       ("JINJA2" "id.MESSAGE" "") ("unknown" "" ":name")))
             (fields (apply append blocks))
             (rows (org-value-plan (cons "contract-block-plans" fields))))
        (check (length rows) => (length blocks))
        (for-each
          (lambda (block row)
            (let ((language (car block)) (name (cadr block)) (parameters (caddr block)))
              (check (list (car row)) => (car (org-value-plan (list "contract-policy" "language" language))))
              (check (list (cadr row)) => (car (org-value-plan (list "contract-policy" "name" name))))
              (check (list (caddr row)) => (car (org-value-plan (list "contract-policy" "named-id" name))))
              (let ((pname (org-value-plan (list "block-parameter" parameters ":name")))
                    (severity (org-value-plan (list "block-parameter" parameters ":severity"))))
                (check (list (list-ref row 3) (list-ref row 4)) =>
                       (if (pair? pname) (list "true" (caar pname)) '("false" "")))
                (check (list (list-ref row 5) (list-ref row 6)) =>
                       (if (pair? severity) (list "true" (caar severity)) '("false" ""))))))
          blocks rows)))
    (test-case "Contract block batches keep count order and absent values"
      (check (org-value-plan '("contract-block-plans")) => '())
      (check (with-catch (lambda (e) #t)
               (lambda () (org-value-plan '("contract-block-plans" "truncated")) #f)) => #t)
      (let* ((one '("org-contract" "中文" ":name message :severity warn"))
             (fields (apply append (make-list 1000 one)))
             (rows (org-value-plan (cons "contract-block-plans" fields))))
        (check (length rows) => 1000)
        (check (andmap (lambda (row) (equal? row '("contract" "中文" "中文" "true" "message" "true" "warn"))) rows) => #t)))))
