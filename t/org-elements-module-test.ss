;;; -*- Gerbil -*-
;;; Org Elements is a POO feature independently of Org Contract.

(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :clan/poo/object .o .ref .slot?)
        (only-in :gerbil-parser/graph-query-support graph-query-context?)
        (only-in "../languages/org/modules/org-elements/types.ss"
                 +org-element-schema+ +org-element-context-kind+
                 org-element-query-context?)
        (only-in "org-elements-test-support.ss"
                 check-org-element-catalog check-org-element-selection
                 check-org-named-query-selection
                 check-org-element-query-aot
                 check-org-headline-properties


                 org-test-form-structured? org-test-source-structured?
                 org-test-sources)
        (only-in "../languages/org/modules/org-elements/headline-properties.ss"

                 todo-word-name
                 todo-open-words
                 todo-done-words
                 todo-state-from-directives

                 todo-keyword-from-directives

                 headline-content-after-todo

                 headline-source-title
                 planning-key-kind
                 headline-display-title
                 headline-anchor-slug
                 headline-comment?
                 priority-token?
                 headline-priority-cookie
                 todo-keyword-matches?
                 memory-headline-state )
        (only-in "../languages/org/modules/org-elements/objects.ss"
                 make-org-headline-properties org-headline-property-field)
        (only-in "../languages/org/modules/org-elements/link-properties.ss"
                 org-image-link?
                 org-link-kind
                 org-link-target-key
                 org-link-protocol
                 org-link-protocol-path
                 org-link-file-path
                 org-link-attachment-path
                 org-link-search
                 org-link-file-path-kind
                 org-link-search-kind
                 org-link-search-value
                 org-expand-link-abbreviation
                 )
        (only-in "../languages/org/modules/org-parser/keyword-funs.ss" org-keyword-facts)
        (only-in "../languages/org/modules/org-elements/logbook-properties.ss"
                 logbook-content-line
                 logbook-line-kind
                 logbook-state-quote-shape
                 logbook-state-to
                 logbook-state-from
                 logbook-clock-duration-shape
                 logbook-clock-duration-value )
        (only-in "../languages/org/modules/org-elements/table-properties.ss"
                 table-column-cookie-match?
                 table-column-cookie-kind )
        (only-in "../languages/org/modules/org-elements/affiliated-properties.ss"
                 org-affiliated-keyword? )
        (only-in "../languages/org/modules/org-elements/catalog.ss" +org-affiliated-keywords+)
        (only-in "../languages/org/modules/org-elements/generated/query-source.ss" org-element-queries)
        (only-in "../languages/org/modules/org-elements/interface.ss"
                 +org-element-kinds+ org-elements-default-profile
                 make-org-element-graph-view make-org-element-query
                 make-org-element-query-context org-element-query?
                 org-element-map org-element-property
                 org-element-lineage? org-element-select
                 org-named-element-query-id org-named-element-query-query
                 org-element-with-headline-properties
                 org-elements property property-contains at child-of))
(export org-elements-module-test)

(def sample-graph
  (make-org-element-graph-view
   (list (.o id: 0 parent: #f kind: "org-data" title: #f)
         (.o id: 1 parent: 0 kind: "headline" title: "Evidence")
         (.o id: 2 parent: 1 kind: "link" title: #f))
   (lambda (record) (.ref record 'id))
   (lambda (record) (.ref record 'parent))
   (lambda (record) (.ref record 'kind))
   (lambda (record name)
     (and (equal? name "title") (.ref record 'title)))))

(def headline-graph
  (make-org-element-graph-view
   (list (.o id: 0 parent: #f kind: "org-data")
         (.o id: 1 parent: 0 kind: "keyword"
             key: "SEQ_TODO" value: "WAIT(w) | DONE(d)")
         (.o id: 2 parent: 0 kind: "headline"
             title: "WAIT [#A] Parent :work:urgent:")
         (.o id: 3 parent: 2 kind: "headline"
             title: "DONE Child :work:")
         (.o id: 4 parent: 0 kind: "headline"
             title: "TODO is ordinary text under this profile")
         (.o id: 5 parent: 0 kind: "headline"
             title: "WAIT :only:")
         (.o id: 6 parent: 0 kind: "headline"
             title: "WAIT Review")
         (.o id: 7 parent: 0 kind: "headline"
             title: "WAIT Audit")
         (.o id: 8 parent: 2 kind: "inlinetask"
             title: "WAIT Inline :work:"))
   (lambda (record) (.ref record 'id))
   (lambda (record) (.ref record 'parent))
   (lambda (record) (.ref record 'kind))
   (lambda (record name)
     (let (slot (string->symbol name))
       (and (.slot? record slot) (.ref record slot))))))

(def org-elements-module-test
  (test-suite "Org Elements POO module"
    (test-case "headline property projection is an admitted POO value"
      (let (properties
             (make-org-headline-properties "TODO A" "A" "TODO" "todo"
                                           "A" '("work")))
        (check (.ref properties 'title) => "A")
        (let-values (((known? value)
                      (org-headline-property-field properties "raw-value")))
          (check known? => #t)
          (check value => "A")))
      (check-exception
       (make-org-headline-properties "A" "A" #f #f #f '(42))
       true))
    (test-case "Scheme AST contract rejects output-based tests"
      (check (org-test-form-structured? '(display "snapshot")) => #f)
      (check (org-test-form-structured? '(string-append "a" "b")) => #f)
      (check (org-test-form-structured? '(quote (display "fixture"))) => #t)
      (check (filter (lambda (path)
                       (not (org-test-source-structured? path)))
                     (org-test-sources "languages/org"))
             => '())
      (check (org-test-source-structured?
              "languages/org/modules/org-elements/headline-properties.ss")
             => #t)
      (check (org-test-source-structured?
              "languages/org/modules/org-elements/link-properties.ss")
             => #t)
      (check (org-test-source-structured?
              "languages/org/modules/org-elements/affiliated-properties.ss")
             => #t))
    (test-case "affiliated keyword membership is Scheme-owned and AOT projected"

      (check (org-affiliated-keyword? "name" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "NAME" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "ATTR_HTML" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "attr_latex" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "TODO" +org-affiliated-keywords+) => #f))
    (test-case "link kind is Scheme-owned and AOT projected"

      (check (org-image-link? "diagram.svg") => #t)
      (check (org-image-link? "diagram.svg?size=2") => #f)
      (check (org-image-link? "https://example.test/doc.org") => #f)











      (check (org-link-kind "*Heading") => "headline")
      (check (org-link-kind "#custom") => "custom-id")
      (check (org-link-kind "id:local") => "id")
      (check (org-link-kind "fn:note") => "footnote")
      (check (org-link-kind "coderef:init") => "code-ref")
      (check (org-link-kind "https://example.org") => "uri")
      (check (org-link-kind "target-one") => "fuzzy")
      (check (org-link-target-key "*Heading") => "Heading")
      (check (org-link-target-key "id:local::*Heading") => "id:local")
      (check (org-link-protocol "https://example.org") => "https")
      (check (org-link-protocol-path "https://example.org") => "//example.org")
      (check (org-link-file-path "file:notes/demo.org::*Heading")
             => "notes/demo.org")
      (check (org-link-attachment-path "attachment:diagram.png::255")
             => "diagram.png")
      (check (org-link-attachment-path "ATTACHMENT:diagram.png")
             => "diagram.png")
      (check (org-link-search "file:notes/demo.org::*Heading")
             => "*Heading")
      (check (org-link-file-path-kind "/tmp/demo.org") => "absolute")
      (check (org-link-file-path-kind "notes/demo.org") => "relative")
      (check (org-link-search-kind "*Heading") => "headline")
      (check (org-link-search-kind "255") => "line-number")
      (check (org-link-search-value "*Heading") => "Heading")
      (check (org-expand-link-abbreviation "https://host/%s" "a/b" "a%2Fb")
             => "https://host/a/b")
      (check (org-expand-link-abbreviation "https://host/%h" "a/b" "a%2Fb")
             => "https://host/a%2Fb")
      (check (org-expand-link-abbreviation "https://host/" "a/b" "a%2Fb")
             => "https://host/a/b"))
    (test-case "document keywords use native batched semantic plans"
      (let (rows (org-keyword-facts "PROPERTY" "  name   value with spaces "))
        (check (member '("first" "name") rows) ? pair?)
        (check (member '("rest" "value with spaces") rows) ? pair?))
      (let (rows (org-keyword-facts "SELECT_TAGS" " alpha  beta\tgamma "))
        (check (filter (lambda (row) (equal? (car row) "word")) rows)
               => '(("word" "alpha") ("word" "beta") ("word" "gamma"))))
      (let (rows (org-keyword-facts "FILETAGS" " :alpha:beta:  gamma : "))
        (check (filter (lambda (row) (equal? (car row) "tag")) rows)
               => '(("tag" "alpha") ("tag" "beta") ("tag" "gamma"))))
      (let (rows (org-keyword-facts "OPTIONS" "H:2 H -:nil e:YES"))
        (check (member '("H" "2") rows) ? pair?)
        (check (member '("-" "false") rows) ? pair?)
        (check (member '("e" "true") rows) ? pair?))
      (let (rows (org-keyword-facts "OPTIONS" "H:2 H: -:maybe e:Nil"))
        (check (member '("H" "") rows) ? pair?)
        (check (member '("-" "") rows) ? pair?)
        (check (member '("e" "false") rows) ? pair?)))
    (test-case "LOGBOOK line shape and kinds are Scheme-owned and AOT projected"

      (check (logbook-content-line "  - State \"DONE\"  ")
             => "State \"DONE\"")
      (check (logbook-content-line " - ") => "")
      (check (logbook-content-line "  CLOCK: [2026-05-14 Thu]  ")
             => "CLOCK: [2026-05-14 Thu]")

      (check (logbook-line-kind "State \"DONE\" from \"TODO\"") => "state")
      (check (logbook-line-kind "Note taken on [2026-05-14 Thu]") => "note")
      (check (logbook-line-kind "Refiled on [2026-05-14 Thu]") => "refile")
      (check (logbook-line-kind "Refiling to [[file:notes.org]]") => "refile")
      (check (logbook-line-kind "Rescheduled from [a] to [b]") => "reschedule")
      (check (logbook-line-kind "New deadline from [a] to [b]") => "redeadline")
      (check (logbook-line-kind "Deadline changed") => "redeadline")
      (check (logbook-line-kind "Removed deadline") => "redeadline")
      (check (logbook-line-kind "CLOCK: [a]--[b]") => "clock")
      (check (logbook-line-kind "unclassified note") => "note"))
    (test-case "LOGBOOK list-item CLOCK duration boundary is Scheme-owned"


      (check (logbook-clock-duration-shape "CLOCK: [a]--[b] => 0:30")
             => "present")
      (check (logbook-clock-duration-value "CLOCK: [a]--[b] => 0:30")
             => "0:30")
      (check (logbook-clock-duration-shape "CLOCK: [a]--[b]")
             => "absent")
      (check (logbook-clock-duration-value "CLOCK: [a] => ") => ""))
    (test-case "LOGBOOK quoted state values are Scheme-owned and AOT projected"



      (check (logbook-state-quote-shape "State \"DONE\" from \"TODO\"")
             => "complete")
      (check (logbook-state-to "State \"DONE\" from \"TODO\"") => "DONE")
      (check (logbook-state-from "State \"DONE\" from \"TODO\"") => "TODO")
      (check (logbook-state-quote-shape "State \"\" from \"\"")
             => "complete")
      (check (logbook-state-to "State \"\" from \"\"") => "")
      (check (logbook-state-from "State \"\" from \"\"") => "")
      (check (logbook-state-quote-shape "State \"DONE\" from \"TODO")
             => "incomplete")
      (check (logbook-state-quote-shape "State DONE from TODO")
             => "incomplete"))
    (test-case "table cookie classification is Scheme-owned and AOT projected"


      (check (table-column-cookie-match? "r3" "r") => #t)
      (check (table-column-cookie-match? "r-1" "r") => #f)
      (check (table-column-cookie-kind " <l> ") => "left")
      (check (table-column-cookie-kind "<c12>") => "center")
      (check (table-column-cookie-kind "<r3>") => "right")
      (check (table-column-cookie-kind "<10>") => "width")
      (check (table-column-cookie-kind "<l>junk>") => "")
      (check (table-column-cookie-kind "<r-1>") => "")
      (check (table-column-cookie-kind "Text") => ""))
    (test-case "catalog and projected query share one feature interface"
      (check-org-element-catalog)
      (check (if (member "headline" +org-element-kinds+) #t #f)
             => #t)
      (check (.ref org-elements-default-profile 'kind) =>
             'org-elements-profile)
      (check (org-element-query? (make-org-element-query "headline" "title" "Evidence"))
             => #t))
    (test-case "map, property, lineage, and scoped selection compose"
      (let* ((context (make-org-element-query-context sample-graph))
             (headlines (org-element-map context "headline"
                                         (lambda (record) #t)))
             (query (org-elements headline (property title "Evidence")
                                  (child-of scope)))
             (reordered (org-elements headline (child-of scope)
                                      (property title "Evidence"))))
        (check (graph-query-context? (.ref context 'index)) => #t)
        (check
         (org-element-query-context?
          (.o kind: +org-element-context-kind+
              schema: +org-element-schema+
              graph: sample-graph
              index: 'legacy))
         => #f)
        (check (length headlines) => 1)
        (check (org-element-property context (car headlines) "title")
               => "Evidence")
        (check (org-element-lineage? context 0 2) => #t)
        (check (org-element-query? reordered) => #t)
        (check-org-element-selection
         query context 0 (list 0)
         (lambda (record) (.ref record 'id)) (list 1))
        (check-org-element-selection
         reordered context 0 (list 0)
         (lambda (record) (.ref record 'id)) (list 1))
        (check-org-element-selection
         (org-elements headline (property-contains title "Evid")
                       (at scope))
         context 1 (list 1)
         (lambda (record) (.ref record 'id)) (list 1))))
    (test-case "query syntax rejects conflicting and unknown declarations"
      (check (org-element-query?
              (org-elements headline (property title "A")
                            (property title "B"))) => #t)
      (check-exception (org-elements invented-kind) true))
    (test-case "tagged named queries use derived headline properties"
      (let* ((graph (org-element-with-headline-properties headline-graph))
             (context (make-org-element-query-context graph))
             (id-of (lambda (record) (.ref record 'id))))
        (check-org-element-query-aot
         org-element-queries
         "languages/org/modules/org-elements/generated/query-pack.rs")
        (check (length org-element-queries) => 5)
        (check-org-named-query-selection
         (car org-element-queries) context 0 '(0) id-of
         "tasks.open" '(2 5 6 7))
        (check-org-named-query-selection
         (cadr org-element-queries) context 0 '(0) id-of
         "tasks.waiting" '(2 5 6 7))
        (check-org-named-query-selection
         (caddr org-element-queries) context 0 '(0) id-of
         "tasks.review-or-audit" '(6 7))
        (check-org-named-query-selection
         (cadddr org-element-queries) context 0 '(0) id-of
         "tasks.done" '(3))
        (check-org-named-query-selection
         (list-ref org-element-queries 4) context 0 '(0) id-of
         "headlines.child" '(3))))
    (test-case "headline properties remain on the Element query graph"




      (check (todo-word-name "WAIT(w@/!)") => "WAIT")
      (check (todo-open-words "NEXT(n) WAIT(w@/!) | DONE (d)")
             => '("NEXT" "WAIT"))
      (check (todo-done-words "NEXT(n) WAIT(w@/!) | DONE (d)")
             => '("DONE" ""))






      (check (headline-source-title "TODO [#A] *Inline* task " "TODO")
             => "*Inline* task ")
      (check (headline-source-title "  plain  " "") => "plain  ")

      (check (planning-key-kind "scheduled") => "scheduled")
      (check (planning-key-kind "CLOSED") => "closed")



      (check (headline-anchor-slug "  Mixed  Case  ") => "mixed-case")
      (check (headline-anchor-slug "RÉSUMÉ Notes") => "résumé-notes")

      (check (headline-comment? "COMMENT Hidden") => #t)
      (check (headline-comment? "COMMENT") => #t)
      (check (headline-comment? "comment Visible") => #f)
      (check (headline-comment? "COMMENTARY Visible") => #f)

      (check (memory-headline-state "todo" #f #f #f) => "current")
      (check (memory-headline-state "done" #f #f #f) => "closed")
      (check (memory-headline-state "" #t #f #f) => "closed")
      (check (memory-headline-state "" #f #t #f) => "current")
      (check (memory-headline-state "" #f #f #f) => "background")
      (check (memory-headline-state "todo" #f #f #t) => "archived")
      (check (priority-token? "[#A]") => #t)
      (check (priority-token? "[#064]") => #t)
      (check (priority-token? "[#65]") => #f)
      (check (priority-token? "[#+1]") => #f)
      (check (priority-token? "[#a]") => #f)
      (check (priority-token? "[#É]") => #f)
      (check (priority-token? "[#A]junk]") => #f)

      (check (headline-priority-cookie "[#A] Parent") => "A")
      (check (headline-priority-cookie "[#064] Parent") => "064")
      (check (headline-priority-cookie "[#65] Parent") => "")
      (check (headline-priority-cookie "Parent") => "")
      (check (headline-display-title
              "[#A] Parent :work:urgent:" #t)
             => "Parent")
      (check (headline-display-title "[#AB] Plan" #f) => "[#AB] Plan")
      (check (headline-display-title "[#65] Plan" #f) => "[#65] Plan")
      (check (headline-display-title "Review" #f) => "Review")
      (check (headline-display-title "Plan :bad::" #f)
             => "Plan :bad::")
      (check (headline-content-after-todo
              "  WAIT   [#A] Parent :work:  " '("WAIT(w) | DONE(d)")
              '("TODO") '("DONE"))
             => "[#A] Parent :work:")
      (check (headline-content-after-todo
              "TODO is ordinary text" '("WAIT(w) | DONE(d)")
              '("TODO") '("DONE"))
             => "TODO is ordinary text")
      (check (todo-keyword-from-directives
              "WAIT Review" '("WAIT(w) | DONE(d)")
              '("TODO") '("DONE")) => "WAIT")
      (check (todo-keyword-from-directives
              "TODO prose" '("WAIT(w) | DONE(d)")
              '("TODO") '("DONE")) => "")
      (check (todo-keyword-matches?
              "WAIT Review" '("WAIT | DONE")
              '("TODO") '("DONE") "WAIT") => #t)
      (check (todo-keyword-matches?
              "WAIT Review" '("HOLD | FINISHED")
              '("TODO") '("DONE") "WAIT") => #f)
      (check (todo-state-from-directives
              "TODO Work" '() '("TODO") '("DONE")) => "todo")
      (check (todo-state-from-directives
              "DONE Work" '() '("TODO") '("DONE")) => "done")
      (check (todo-state-from-directives
              "WAIT Work" '() '("WAIT") '("FINISHED")) => "todo")
      (check (todo-state-from-directives
              "FINISHED Work" '() '("WAIT") '("FINISHED")) => "done")
      (check (todo-state-from-directives
              "WAIT Work" '("HOLD | DONE") '("WAIT") '("FINISHED"))
             => "")
      (check (todo-state-from-directives "WAIT Work"
                                         '("WAIT(w) | DONE(d)")
                                         '("TODO") '("DONE")) => "todo")
      (check (todo-state-from-directives "FINISHED Work"
                                         '("WAIT(w) | DONE(d)"
                                           "HOLD(h) | FINISHED(f)")
                                         '("TODO") '("DONE")) => "done")
      (let* ((graph (org-element-with-headline-properties
                     sample-graph '("Evidence") '("FINISHED")))
             (context (make-org-element-query-context graph))
             (headline (car (org-element-map context "headline"
                                              (lambda (record) #t)))))
        (check (org-element-property context headline "todo-type") => "todo")
        (check (org-element-property context headline "todo-keyword")
               => "Evidence"))
      (let* ((graph (org-element-with-headline-properties headline-graph))
             (context (make-org-element-query-context graph))
             (records (org-element-map context "headline"
                                       (lambda (record) #t)))
             (parent (car records))
             (child (cadr records))
             (plain (caddr records))
             (tag-only (cadddr records))
             (inline (car (org-element-map context "inlinetask"
                                           (lambda (record) #t)))))
        (check-org-headline-properties
         context parent "WAIT [#A] Parent :work:urgent:"
         "Parent" "WAIT" "todo" "A"
         '("work" "urgent"))
        (check-org-headline-properties
         context child "DONE Child :work:"
         "Child" "DONE" "done" #f '("work"))
        (check-org-headline-properties
         context plain "TODO is ordinary text under this profile"
         "TODO is ordinary text under this profile"
         #f #f #f '())
        (check-org-headline-properties
         context tag-only "WAIT :only:" "" "WAIT" "todo" #f '("only"))
        (check-org-headline-properties
         context inline "WAIT Inline :work:"
         "Inline" "WAIT" "todo" #f '("work"))
        (check (org-element-lineage? context 2 3) => #t)
        (check-org-element-selection
         (org-elements headline (property todo-keyword "WAIT"))
         context 0 (list 0)
         (lambda (record) (.ref record 'id)) (list 2 5 6 7))
        (check-org-element-selection
         (org-elements headline (property tags "work"))
         context 0 (list 0)
         (lambda (record) (.ref record 'id)) (list 2 3))
        (check-org-element-selection
         (org-elements headline (property todo-type "done")
                       (child-of scope))
         context 2 (list 2)
         (lambda (record) (.ref record 'id)) (list 3))
        (check-org-element-selection
         (org-elements inlinetask (property todo-type "todo")
                       (child-of scope))
         context 2 (list 2)
         (lambda (record) (.ref record 'id)) (list 8))))))
