;;; -*- Gerbil -*-
(import (only-in :std/test check check-exception test-case test-suite)
        "../languages/org/modules/org-parser/text-funs.ss"
        "../languages/org/modules/org-parser/block-line-funs.ss"
        "../languages/org/modules/org-parser/keyword-funs.ss"
        "../languages/org/modules/org-parser/dir-path-funs.ss"
        "../languages/org/modules/org-parser/family-funs.ss"
        (only-in "../languages/org/modules/org-elements/radio-match.ss" org-radio-matches)
        (only-in "../languages/org/modules/org-contract/document-plan.ss" org-contract-document-plan)
        (only-in "../languages/org/native-event-tape.ss"
                 org-request->tape expression-value-rows expectation-value-rows contract-value-rows))
(export org-native-semantic-owner-test)

(def (contract-plan fields)
  (org-contract-document-plan fields expression-value-rows expectation-value-rows contract-value-rows))

(def (semantic-request operation fields)
  (let* ((payloads (map string->utf8 fields))
         (size (+ 18 (foldl (lambda (payload total) (+ total 4 (u8vector-length payload))) 0 payloads)))
         (bytes (make-u8vector size 0)))
    (u8vector-copy! bytes 0 (u8vector 79 78 82 49))
    (u8vector-set! bytes 4 operation)
    (def (put! offset value)
      (for-each (lambda (index) (u8vector-set! bytes (+ offset index)
                                            (modulo (quotient value (expt 256 index)) 256))) '(0 1 2 3)))
    (put! 14 (length fields))
    (let write ((rest payloads) (offset 18))
      (unless (null? rest)
        (let* ((payload (car rest)) (size (u8vector-length payload)))
          (put! offset size)
          (u8vector-copy! bytes (+ offset 4) payload)
          (write (cdr rest) (+ offset 4 size)))))
    bytes))

(def org-native-semantic-owner-test
  (test-suite "Native semantic owner closure"
    (test-case "Contract document properties use native last wins and policy defaults"
      (let (rows (contract-plan '("1" "4" "CONTRACT_ID" " old " "contract_id" " λ "
                                 "CONTRACT_ALIAS" "a,b a" "SEVERITY" "warning" "0")))
        (check (caddr (car rows)) => "true")
        (check (list-ref (car rows) 3) => "λ")
        (check (list-tail (car rows) 14) => '("org-elements" "subtree" "warning"))
        (check (cdr rows) => '(("alias" "0" "a") ("alias" "0" "b") ("alias" "0" "a"))))
      (check (list-tail (car (contract-plan '("1" "3" "CONTRACT_KIND" "bad"
                                             "CONTRACT_SCOPE" "" "SEVERITY" "bad" "0"))) 14)
             => '("other" "other" "")))
    (test-case "Contract document cooked streams isolate malformed source and reject framing"
      (let* ((fields '("0" "3" "org-elements-query" "" "" "("
                       "org-elements-selector" "" "" "(:org-element (:type paragraph))"
                       "org-elements-expect" "" "" "count >= 1"))
             (rows (contract-plan fields)))
        (check (member '("forms" "0" "query" "invalid") rows) ? pair?)
        (check (member '("forms" "1" "query" "ok") rows) ? pair?)
        (check (member '("forms" "2" "expect" "ok") rows) ? pair?)
        (check (subu8vector (org-request->tape
                            (semantic-request 23 (cons "contract-document-plan" fields))) 0 4)
               => (u8vector 79 88 86 49)))
      (for-each (lambda (fields) (check-exception (contract-plan fields) true))
                '(("01" "0" "0") ("0" "1" "org-elements-query")
                  ("0" "0" "trailing") ("1" "1" "CONTRACT_ID"))))
    (test-case "Contract document counted metadata scales without argument expansion"
      (for-each
       (lambda (count)
         (let* ((fields (cons "0" (cons (number->string count)
                                        (foldl (lambda (i out)
                                                 (cons "text" (cons "" (cons "" (cons "λ" out)))))
                                               '() (iota count)))))
                (rows (contract-plan fields)))
           (check (length rows) => count)
           (check (cadar rows) => "0")
           (check (cadr (list-ref rows (- count 1))) => (number->string (- count 1)))))
       '(1000 10000)))
    (test-case "closed value families reject malformed counts and booleans"
      (check (org-family-call '("planning-key-kind" "closed")) => '(("closed")))
      (check (org-affiliation-batch '("1" "NAME" "3" "name" "ATTR_HTML" "TODO")) => '(("true") ("true") ("false")))
      (check (org-table-batch '("<l10>" "<9223372036854775808>")) => '(("left") (""))))
    (test-case "headline and link batches retain ten thousand framed values"
      (for-each
       (lambda (count)
         (let* ((titles (foldl (lambda (i out) (cons "TODO [#A] λ :work:" (cons "TODO [#A] λ " (cons "true" out)))) '() (iota count)))
                (fields (append '("1" "TODO" "1" "DONE" "0") (cons (number->string (* 3 count)) titles)))
                (rows (org-document-batch fields))
                (tape (org-request->tape (semantic-request 17 fields))))
           (check (length rows) => (+ count 1))
           (check (cadr rows) => '("todo" "TODO" "[#A] λ :work:" "A" "λ" "λ " "false"))
           (check (subu8vector tape 0 4) => (u8vector 79 88 86 49)))
         (check (length (org-link-batch (make-list count "file:λ.org::*Heading"))) => count))
       '(1000 10000)))
    (test-case "radio matching is native indexed and byte ranged"
      (check (org-radio-matches "é Alpha Beta Alpha Alphabet Alpha-Beta" '("Alpha" "Alpha Beta" "Alpha"))
             => '(("3" "13" "1") ("14" "19" "0")))
      (check (length (org-radio-matches (string-join (make-list 10000 "é Alpha ") "") '("Alpha"))) => 10000))
    (test-case "physical lines retain UTF-8 offsets and all terminators"
      (check (org-physical-lines "λ\r\nβ\rγ\n") => '(("λ" "\r\n" 0 2) ("β" "\r" 4 6) ("γ" "\n" 7 9)))
      (check (org-physical-lines "") => '())
      (check (org-physical-lines "\n") => '(("" "\n" 0 0)))
      (check (org-physical-lines "α") => '(("α" "" 0 2))))
    (test-case "block normalization code references and blank minimum are native"
      (let* ((a '("\tλ (ref:ok)\r\n" "\tλ (ref:ok)\r\n" "(ref:%s)" "4" "false"))
             (empty '("" "" "(ref:%s)" "4" "false"))
             (rows (org-block-document-plan (append a empty a))))
        (check (car rows) => '("block" "1"))
        (check (cadr rows) => (car (org-block-line-facts (car a) (cadr a) (caddr a) 4 #f)))
        (check (list-ref rows 2) => '("block" "0"))
        (check (list-ref rows 3) => '("block" "1"))
        (check (list-ref rows 4) => (cadr rows)))
      (check-exception (org-block-document-plan '("body")) true)
      (check-exception (org-block-document-plan '("" "" "%s" "-1" "false")) true)
      (check-exception (org-block-document-plan '("" "" "%s" "4" "maybe")) true)
      (check (org-family-call '("headline-document-anchors" "5" "Same" "Same" "" "Other" "Same"))
             => '(("same" "same-1" "" "other" "same-2")))
      (let (rows (org-block-line-facts "\tλ (ref:ok)\r\n\tβ\r" ",\tλ (ref:ok)\r\n\tβ\r" "(ref:%s)" 4 #f))
        (check (length rows) => 2)
        (check (list-ref (car rows) 3) => "λ (ref:ok)")
        (check (list-ref (car rows) 5) => "λ")
        (check (list-ref (car rows) 10) => "4")
        (check (list-ref (car rows) 11) => "12"))
      (check (map (lambda (row) (list-ref row 6))
                  (org-block-line-facts "  α\n\n  β" "  α\n\n  β" "(ref:%s)" 4 #f))
             => '("0" "0" "0"))
      (check (org-dynamic-content-facts "open\r\n \r\n") => '("1" "false"))
      (check (org-dynamic-content-facts "open\r\nλ\r\n\r\n") => '("2" "true")))
    (test-case "large blocks never expand one procedure argument per line"
      (for-each
       (lambda (count)
         (let* ((text (string-join (make-list count "  λ (ref:x)\r\n") ""))
                (rows (org-block-line-facts text text "(ref:%s)" 4 #f)))
           (check (length rows) => count)
           (check (list-ref (car rows) 6) => "2")))
       '(1000 10000)))
    (test-case "large block tape and keyword batches retain counted framing"
      (for-each
       (lambda (count)
         (let* ((text (string-join (make-list count "  λ (ref:x)\r\n") ""))
                (tape (org-request->tape (semantic-request 10 (list text text "(ref:%s)" "4" "false")))))
           (check (subu8vector tape 0 4) => (u8vector 79 88 86 49)))
         (let (tape (org-request->tape
                    (semantic-request 12 (foldl (lambda (index fields) (cons "PROPERTY" (cons "X λ" fields)))
                                               '() (iota count)))))
           (check (subu8vector tape 0 4) => (u8vector 79 88 86 49))))
       '(1000 10000)))
    (test-case "keyword plans retain last option wins and ASCII case ownership"
      (let* ((fields '("0" "0" "4" "options" "H:2 H:3 -:nil e:YES" "FILETAGS" ":α:β:α:" "0"))
             (facts (org-document-batch fields)))
        (check facts => (append '(("0" "0") ("keyword" "0" "10"))
                               (org-keyword-facts "options" "H:2 H:3 -:nil e:YES")
                               '(("keyword" "1" "6"))
                               (org-keyword-facts "FILETAGS" ":α:β:α:")))
        (check (subu8vector (org-request->tape (semantic-request 17 fields)) 0 4)
               => (u8vector 79 88 86 49)))
      (let (rows (org-keyword-facts "options" "H:2 H:3 -:nil e:YES"))
        (check (member '("route" "OPTIONS") rows) ? pair?)
        (check (member '("H" "3") rows) ? pair?)
        (check (member '("-" "false") rows) ? pair?)
        (check (member '("e" "true") rows) ? pair?))
      (check (car (org-keyword-facts "lınk" "x y")) => '("route" ""))
      (check (filter (lambda (row) (string=? (car row) "tag"))
                      (org-keyword-facts "FILETAGS" ":α:β:α:"))
             => '(("tag" "α") ("tag" "β") ("tag" "α"))))
    (test-case "DIR plans preserve quote nesting malformed literals and effect boundaries"
      (check (org-dir-command-plan " λ/$(printf ')')/$(echo (nested))/$(unclosed")
             => '(("literal" " λ/" "") ("command" "printf ')'" "$(printf ')')")
                  ("literal" "/" "") ("command" "echo (nested)" "$(echo (nested))")
                  ("literal" "/$(unclosed" "")))
      (check (org-dir-command-plan "$(printf \"x)y\")")
             => '(("command" "printf \"x)y\"" "$(printf \"x)y\")")))
      (check (org-dir-command-plan "$(printf x\\)y)")
             => '(("command" "printf x\\)y" "$(printf x\\)y)")))
      (check (org-dir-environment-plan "$A-._1/$(bad")
             => '(("environment" "A-._1" "$A-._1") ("literal" "/$(bad" ""))))))
