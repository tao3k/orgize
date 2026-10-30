;;; -*- Gerbil -*-
(import (only-in :std/test check test-case test-suite)
        (only-in "../languages/org/modules/org-parser/value-funs.ss" org-value-plan))
(export org-metadata-value-test)
(def org-metadata-value-test
  (test-suite "Native downstream metadata and Agenda grammar"
    (test-case "Calendar date values admit leap years and reject malformed or out of range parts"
      (check (org-value-plan '("agenda-date" "2024-02-29")) => '(("2024" "2" "29")))
      (check (org-value-plan '("agenda-date" "2025-02-29")) => '())
      (check (org-value-plan '("agenda-date" "65536-01-01")) => '())
      (check (org-value-plan '("agenda-date" "2026-10-06-extra")) => '()))
    (test-case "SDD parent values preserve optional target and label fields"
      (check (org-value-plan '("sdd-parent-reference" " [[id:parent][Label]] ")) =>
             '(("[[id:parent][Label]]" "parent" "Label")))
      (check (org-value-plan '("sdd-parent-reference" "[[id:][ ]]")) => '(("[[id:][ ]]" "" "")))
      (check (org-value-plan '("sdd-parent-reference" "other")) => '(("other" "" "")))
      (check (org-value-plan '("sdd-parent-reference" "  ")) => '()))
    (test-case "Workspace key and protocol roles are native values"
      (check (org-value-plan '("workspace-link-key" " id:parent::search ")) => '(("id:parent")))
      (check (org-value-plan '("workspace-link-key" "https://external")) => '())
      (check (org-value-plan '("workspace-link-role" "#custom")) => '(("custom-id")))
      (check (org-value-plan '("workspace-link-role" "coderef:code")) => '(("coderef"))))
    (test-case "Interactive choices preserve typed rows and source diagnostics without evaluation"
      (check (org-value-plan (list "interactive-choice"
               "id: one\nmethod: choice\nstage: review\ngroup: -\ninfo: select\ncategories: 1=one,?=detail\ndetails:\n| n | id | contract | full | use-if |\n| 1 | one | - | Full | Always |\n")) =>
             '(("choice" "one" "choice" "review" "" "" "" "select")
               ("category" "1" "one" "false") ("category" "?" "detail" "true")
               ("entry" "1" "one" "" "Full" "Always")))
      (check (org-value-plan '("interactive-choice" "")) => '(("error" "agent-interactive choice requires `id`")))
      (check (org-value-plan (list "interactive-choice"
               "id: one\nmethod: choice\nstage: review\ninfo: select\ncategories: 1=missing,?=detail\ndetails:\n| 1 | one | - | Full | Always |\n")) =>
             '(("error" "agent-interactive `one` category `1=missing` must match a detail row"))))
    (test-case "Datetree prefixes preserve bounds without calendar interpretation"
      (check (org-value-plan '("datetree-title" "year" "2026 Year")) => '(("2026")))
      (check (org-value-plan '("datetree-title" "day" "2026-99-00 title")) => '(("2026" "99" "0")))
      (check (org-value-plan '("datetree-title" "month" " 2026-10")) => '())
      (check (org-value-plan '("datetree-title" "day" "2026-1x-06")) => '()))
    (test-case "Timestamp ordering handles Unicode and delimiter boundaries"
      (check (org-value-plan '("timestamp-sort-key" "日期 <2026-10-06 Tue 09:30>")) => '(("2026" "10" "6" "9" "30")))
      (check (org-value-plan '("timestamp-sort-key" "[2026-10-06] 09:30")) => '(("2026" "10" "6" "0" "0")))
      (check (org-value-plan '("timestamp-sort-key" "not a date")) => '()))
    (test-case "Columns preserve empty title presence and inert summaries"
      (check (org-value-plan '("column-summary-kind" "X%")) => '(("checkbox-percent")))
      (check (org-value-plan '("column-summary-kind" "x%")) => '(("custom")))
      (check (org-value-plan '("column-values" "%20item() %effort{+;%.2f} ignored")) =>
             '(("ITEM" "" "true" "20" "" "" "%20item()")
               ("EFFORT" "" "false" "" "+" "%.2f" "%effort{+;%.2f}")))
      (check (org-value-plan '("column-precision" " %.2f ")) => '(("2")))
      (check (org-value-plan '("column-precision" "%.xf")) => '())
      (check (org-value-plan '("checkbox-done" "[2/2]")) => '(("true")))
      (check (org-value-plan '("checkbox-done" "[0/0]")) => '(("false")))
      (check (org-value-plan '("checkbox-done" "[18446744073709551616/1]")) => '(("false"))))
    (test-case "Clock scopes are classified without host execution"
      (check (org-value-plan '("clock-tree-level" "tree+4")) => '(("4")))
      (check (org-value-plan '("clock-tree-level" "treeX")) => '())
      (check (org-value-plan '("clock-scope" "\"TREE4\"")) => '(("tree-level")))
      (check (org-value-plan '("clock-scope" "tree+4")) => '(("tree-level")))
      (check (org-value-plan '("clock-scope" "")) => '(("unknown")))
      (check (org-value-plan '("clock-scope" "(system dangerous)")) => '(("external"))))
    (test-case "Property schema references remain inert"
      (check (org-value-plan '("property-schema-reference" "[[file:a.org][Label]]")) =>
             '(("[[file:a.org][Label]]" "file:a.org" "org-file-link")))
      (check (org-value-plan '("property-schema-reference" "{{{schema(./a.json)}}}")) =>
             '(("{{{schema(./a.json)}}}" "./a.json" "macro")))
      (check (org-value-plan '("property-schema-reference" "schema-id")) =>
             '(("schema-id" "schema-id" "contract-id"))))
    (test-case "Agenda grammar preserves signs operators quotes patterns and UTF8 offsets"
      (check (org-value-plan '("agenda-match" "+work-priority|Effort<=*2&OWNER=\"a|b\"&TAG={x+y}")) =>
             '(("source" "+work-priority|Effort<=*2&OWNER=\"a|b\"&TAG={x+y}")
               ("term" "0" "true" "tag" "work" "" "" "")
               ("term" "0" "false" "tag" "priority" "" "" "")
               ("term" "1" "true" "property" "Effort" "le" "bare" "2")
               ("term" "1" "true" "property" "OWNER" "eq" "quoted" "a|b")
               ("term" "1" "true" "property" "TAG" "eq" "pattern" "x+y")))
      (check (org-value-plan '("agenda-match" "名称=")) => '(("error" "7" "property match is missing a value")))
      (check (org-value-plan '("agenda-match" "a||b")) => '(("error" "2" "match clause is empty")))
      (check (org-value-plan '("agenda-match" "  ")) => '(("error" "0" "match expression is empty")))
      (check (org-value-plan '("agenda-match" "ok")) =>
             '(("source" "ok") ("term" "0" "true" "tag" "ok" "" "" ""))))))
