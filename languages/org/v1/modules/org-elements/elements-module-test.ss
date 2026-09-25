;;; -*- Gerbil -*-
;;; Org Elements is a POO feature independently of Org Contract.

(import (only-in :std/test check check-exception test-case test-suite)
        (only-in :clan/poo/object .o .ref .slot?)
        (only-in "test-syntax.ss"
                 check-org-element-catalog check-org-element-selection
                 check-org-named-query-selection
                 check-org-element-query-aot
                 check-org-headline-properties
                 check-org-headline-ir
                 check-org-headline-state-aot
                 org-test-form-structured? org-test-source-structured?
                 org-test-sources)
        (only-in "headline-properties.ss"
                 todo-directive-rust
                 todo-state-from-directives
                 todo-state-from-directives-rust
                 todo-keyword-from-directives
                 todo-keyword-from-directives-rust
                 headline-content-after-todo
                 headline-content-after-todo-rust
                 headline-display-title headline-display-title-rust
                 todo-keyword-matches? todo-keyword-matches-rust)
        (only-in "link-properties.ss"
                 org-image-link? org-image-link-rust)
        (only-in "generated/query-source.ss" org-element-queries)
        (only-in "interface.ss"
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
             title: "WAIT Audit"))
   (lambda (record) (.ref record 'id))
   (lambda (record) (.ref record 'parent))
   (lambda (record) (.ref record 'kind))
   (lambda (record name)
     (let (slot (string->symbol name))
       (and (.slot? record slot) (.ref record slot))))))

(def org-elements-module-test
  (test-suite "Org Elements POO module"
    (test-case "Scheme AST contract rejects output-based tests"
      (check (org-test-form-structured? '(display "snapshot")) => #f)
      (check (org-test-form-structured? '(string-append "a" "b")) => #f)
      (check (org-test-form-structured? '(quote (display "fixture"))) => #t)
      (check (filter (lambda (path)
                       (not (org-test-source-structured? path)))
                     (org-test-sources "languages/org/v1"))
             => '())
      (check (org-test-source-structured?
              "languages/org/v1/modules/org-elements/generate-headline-ir.ss")
             => #t)
      (check (org-test-source-structured?
              "languages/org/v1/modules/org-elements/headline-properties.ss")
             => #t)
      (check (org-test-source-structured?
              "languages/org/v1/modules/org-elements/link-properties.ss")
             => #t))
    (test-case "link kind is Scheme-owned and AOT projected"
      (check-org-headline-ir
       org-image-link-rust 'org_image_link_p
       "languages/org/v1/modules/org-elements/generated/org_image_link_p.ir.json")
      (check (org-image-link? "diagram.svg") => #t)
      (check (org-image-link? "diagram.svg?size=2") => #f)
      (check (org-image-link? "https://example.test/doc.org") => #f))
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
         "languages/org/v1/modules/org-elements/generated/query-pack.rs")
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
      (check-org-headline-ir
       todo-directive-rust 'todo_directive_p
       "languages/org/v1/modules/org-elements/generated/todo_directive_p.ir.json")
      (check-org-headline-ir
       todo-state-from-directives-rust 'todo_state_from_directives
       "languages/org/v1/modules/org-elements/generated/todo_state_from_directives.ir.json")
      (check-org-headline-state-aot todo-state-from-directives-rust)
      (check-org-headline-ir
       todo-keyword-matches-rust 'todo_keyword_matches_p
       "languages/org/v1/modules/org-elements/generated/todo_keyword_matches_p.ir.json")
      (check-org-headline-ir
       todo-keyword-from-directives-rust 'todo_keyword_from_directives
       "languages/org/v1/modules/org-elements/generated/todo_keyword_from_directives.ir.json")
      (check-org-headline-ir
       headline-content-after-todo-rust 'headline_content_after_todo
       "languages/org/v1/modules/org-elements/generated/headline_content_after_todo.ir.json")
      (check-org-headline-ir
       headline-display-title-rust 'headline_display_title
       "languages/org/v1/modules/org-elements/generated/headline_display_title.ir.json")
      (check (headline-display-title
              "[#A] Parent :work:urgent:")
             => "Parent")
      (check (headline-display-title "Review") => "Review")
      (check (headline-content-after-todo
              "  WAIT   [#A] Parent :work:  " '("WAIT(w) | DONE(d)"))
             => "[#A] Parent :work:")
      (check (headline-content-after-todo
              "TODO is ordinary text" '("WAIT(w) | DONE(d)"))
             => "TODO is ordinary text")
      (check (todo-keyword-from-directives
              "WAIT Review" '("WAIT(w) | DONE(d)")) => "WAIT")
      (check (todo-keyword-from-directives
              "TODO prose" '("WAIT(w) | DONE(d)")) => "")
      (check (todo-keyword-matches?
              "WAIT Review" '("WAIT | DONE") "WAIT") => #t)
      (check (todo-keyword-matches?
              "WAIT Review" '("HOLD | FINISHED") "WAIT") => #f)
      (check (todo-state-from-directives "TODO Work" '()) => "todo")
      (check (todo-state-from-directives "DONE Work" '()) => "done")
      (check (todo-state-from-directives "WAIT Work"
                                         '("WAIT(w) | DONE(d)")) => "todo")
      (check (todo-state-from-directives "FINISHED Work"
                                         '("WAIT(w) | DONE(d)"
                                           "HOLD(h) | FINISHED(f)")) => "done")
      (let* ((graph (org-element-with-headline-properties headline-graph))
             (context (make-org-element-query-context graph))
             (records (org-element-map context "headline"
                                       (lambda (record) #t)))
             (parent (car records))
             (child (cadr records))
             (plain (caddr records))
             (tag-only (cadddr records)))
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
         (lambda (record) (.ref record 'id)) (list 3))))))
