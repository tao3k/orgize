;;; -*- Gerbil -*-
(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/modules/org-parser/value-funs.ss" org-value-plan))
(export org-lifecycle-value-test)
(def org-lifecycle-value-test
  (test-suite "Native lifecycle and metadata value closure"
    (test-case "priority canonical domain and Unicode trim"
      (check (org-value-plan '("priority" " A ")) => '(("letter" "A")))
      (check (org-value-plan '("priority" "64")) => '(("numeric" "64")))
      (for-each (lambda (text) (check (org-value-plan (list "priority" text)) => '()))
                '("00" "064" "65" "a" "AB" "")))
    (test-case "duration units HMS decimals and saturation"
      (for-each
       (lambda (raw)
         (let (scalar (org-value-plan (list "duration" raw)))
           (check (org-value-plan (list "duration-plan" raw))
                  => (if (null? scalar) '(("false" ""))
                       (list (cons "true" (car scalar)))))))
       '("" "1:30" "1:02:03" "2h" "1d3h5min" "1y" "1e308" "-1" "λ" "NaN"))
      (check (org-value-plan '("duration-plan" "1:30" "λ" ""))
             => '(("true" "5400") ("false" "") ("true" "0")))
      (for-each
       (lambda (example)
         (check (org-value-plan (list "duration" (car example))) => (list (list (cadr example)))))
       '(("" "0") ("1:30" "5400") ("1:02:03" "3723") ("2h" "7200")
         ("1d3h5min" "97500") ("1h 0:30" "5400") ("1y" "31557600")
         ("1e2" "6000") (".5" "30") ("0.008333333333333333" "1")
         ("1:99:99" "9639") ("1e308" "18446744073709551615")))
      (for-each (lambda (text) (check (org-value-plan (list "duration" text)) => '()))
                '("-0" "-1" "+inf" "NaN" "inf" "1e999" "1/2" "#x10" "1h junk" "1:2")))
    (test-case "archive empty sides and first separator"
      (check (org-value-plan '("archive" " file.org :: Heading::Nested ")) =>
             '(("file.org :: Heading::Nested" "file.org" "Heading::Nested")))
      (check (org-value-plan '("archive" "::")) => '(("::" "" ""))))
    (test-case "headline time native ASCII boundaries and timestamps"
      (check (org-value-plan '("headline-time" "<2026-10-05 Mon 08:00> meet 9am--10:30pm")) =>
             '(("9" "0" "22" "30")))
      (check (org-value-plan '("headline-time" "λ12am")) => '(("0" "0")))
      (check (org-value-plan '("headline-time" "x9am 29:59")) => '(("29" "59")))
      (check (org-value-plan '("headline-time" "99:00 9:60")) => '()))
    (test-case "timer signs bounds minute admission and overflow"
      (check (org-value-plan '("timer-stamps" "λ +1:02:03 -0:01:00 x2:00:00 3:60:00")) =>
             '(("+1:02:03" "3723") ("-0:01:00" "-60")))
      (check (org-value-plan '("timer-stamps" "9223372036854775807:00:00")) => '()))
    (test-case "metadata words and LINK native normalization"
      (check (org-value-plan '("feed-status" "  ((\"A\" x))\r\n\n  ((\"B\" y))  ")) =>
             '(("((\"A\" x))\n((\"B\" y))" "true" "2")))
      (check (org-value-plan '("link-abbreviation-index" "HtTp" "ftp" "HTTP" "http")) =>
             '(("1")))
      (check (org-value-plan '("words" " A\tB\nλ ")) => '(("A" "B" "λ")))
      (check (org-value-plan '("ascii-lower" "HTTPİ")) => '(("httpİ")))
      (check (org-value-plan '("link-search-normalize" "regexp" "///ΣΟΣİ///" "")) =>
             '(("σοσi̇")))
      (check (org-value-plan '("link-search-normalize" "text" "" "ΣΟΣ")) =>
             '(("σος"))))
    (test-case "clocktable windows preserve leap weeks and absolute bounds"
      (check (org-value-plan '("clock-bound" "1970-01-01" "start")) => '(("1970" "1" "1" "0" "0" "0")))
      (check (org-value-plan '("clock-window" "1970-Q1")) => '(("1970" "1" "1" "0" "0" "0" "1970" "4" "1" "0" "0" "129600")))
      (check (org-value-plan '("clock-window" "2021-W53")) => '())
      (check (org-value-plan '("clock-bound" "2026-02-29" "start")) => '()))
    (test-case "property and clocktable tokens retain quote policies"
      (check (org-value-plan '("property-tokens" "\"\" 'two words' escaped\\ value tail\\")) => '(("" "two words" "escaped value" "tail\\")))
      (check (org-value-plan '("clock-property-names" "'(EFFORT \"CUSTOM NAME\")")) => '(("EFFORT" "CUSTOM NAME")))
      (check (org-value-plan '("clock-property-names" "'nil")) => '())
      (check (org-value-plan '("clock-property-names" "NIL")) => '(())))
    (test-case "progress descriptor and tag values are native"
      (check (org-value-plan '("statistic-cookie" "[[ +01 / +02 ]]")) => '(("fraction" "1" "2" "")))
      (check (org-value-plan '("statistic-cookie" "[101%]")) => '(("unknown" "" "" "")))
      (check (org-value-plan '("descriptor-name" "effort_ALL+++")) => '(("effort")))
      (check (org-value-plan '("tag-values" ":a::b:")) => '(("a" "b"))))
    (test-case "org protocol intent is inert and parameter presence exact"
      (check (org-value-plan '("link-protocol-kind" "shell" "false")) => '(("executable")))
      (check (org-value-plan '("org-protocol" "ORG-PROTOCOL://capture?key=&flag&x=a%20b" "raw")) =>
             '(("capture" "capture") ("key=" "key" "" "true") ("flag" "flag" "" "false") ("x=a%20b" "x" "a%20b" "true"))))
    (test-case "large time and duration plans retain bounded scanning"
      (let (source (string-join (make-list 10000 "0:00:01") " "))
        (check (length (org-value-plan (list "timer-stamps" source))) => 10000))
      (check (org-value-plan (list "headline-time" (string-append (make-string 10000 #\x) " 9am"))) =>
             '(("9" "0"))))))
