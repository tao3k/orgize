;;; -*- Gerbil -*-
(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/v1/modules/org-parser/value-funs.ss" org-value-plan))
(export org-native-publishing-value-test)
(def (plan key raw) (org-value-plan (list "publishing-keyword" key raw)))
(def org-native-publishing-value-test
  (test-suite "Native publishing policy"
    (test-case "Keyword roles are ASCII insensitive and values preserve Unicode"
      (check (plan "Export_File_Name" "　路径.html ") => '(("export-file-name" "路径.html")))
      (check (plan "setupfile" " a ") => '(("setup-file" "a")))
      (check (plan "HtMl_HEAD" " <b> ") => '(("backend-keyword" "<b>")))
      (check (plan "EXPORT_TITLE" " title ") => '(("backend-keyword" "title")))
      (check (plan "TITLE" "title") => '())
      (check (plan "HTML" "title") => '()))
    (test-case "BIND is inert text with Unicode whitespace and optional value"
      (check (plan "bind" "　name (lambda () (error x))　") =>
             '(("bind" "name" "(lambda () (error x))")))
      (check (plan "BIND" " name ") => '(("bind" "name" "")))
      (check (plan "BIND" "　 ") => '()))
    (test-case "OPTIONS preserve first colon, empty fields, order and case"
      (check (plan "options" " H:3　num:t :empty key: key:a:b bad H:4 h:2 ") =>
             '(("option" "H" "3" "H:3" "H")
               ("option" "num" "t" "num:t" "num")
               ("option" "" "empty" ":empty" "other")
               ("option" "key" "" "key:" "other")
               ("option" "key" "a:b" "key:a:b" "other")
               ("option" "H" "4" "H:4" "H")
               ("option" "h" "2" "h:2" "other"))))
    (test-case "Attribute backend policy preserves legacy repeated prefixes"
      (check (plan "ATTR_HTML" ":class cover") => '(("attribute" "html")))
      (check (plan "attr_attr_LaTeX" "") => '(("attribute" "latex")))
      (check (plan "ATTR_" "") => '(("attribute" ""))))
    (test-case "Every closed option kind survives native admission"
      (for-each
        (lambda (key)
          (check (plan "OPTIONS" (string-append key ":t")) =>
                 (list (list "option" key "t" (string-append key ":t") key))))
        '("H" "num" "-" "e" "todo" "tags" "<" "author" "creator"
          "date" "email" "title" "d" "p" "pri" "broken-links")))
    (test-case "Batch rows retain keyword indexes and source order"
      (check (org-value-plan '("publishing-keywords" "TITLE" "ignored"
                "OPTIONS" "H:3 H:4" "BIND" " name nil" "EXPORT_FILE_NAME" "last")) =>
             '(("1" "option" "H" "3" "H:3" "H")
               ("1" "option" "H" "4" "H:4" "H")
               ("2" "bind" "name" "nil")
               ("3" "export-file-name" "last")))
      (check (org-value-plan '("publishing-keywords")) => '()))))
