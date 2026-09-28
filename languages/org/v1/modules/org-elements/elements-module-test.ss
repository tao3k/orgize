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
                 todo-word-name todo-word-name-rust
                 todo-open-words todo-open-words-rust
                 todo-done-words todo-done-words-rust
                 todo-state-from-directives
                 todo-state-from-directives-rust
                 todo-keyword-from-directives
                 todo-keyword-from-directives-rust
                 headline-content-after-todo
                 headline-content-after-todo-rust
                 headline-source-title headline-source-title-rust
                 planning-key-kind planning-key-kind-rust
                 headline-display-title headline-display-title-rust
                 headline-anchor-slug headline-anchor-slug-rust
                 headline-comment? headline-comment-rust
                 priority-token? priority-token-rust
                 headline-priority-cookie headline-priority-cookie-rust
                 todo-keyword-matches? todo-keyword-matches-rust
                 memory-headline-state memory-headline-state-rust)
        (only-in "objects.ss"
                 make-org-headline-properties org-headline-property-field)
        (only-in "link-properties.ss"
                 org-image-link? org-image-link-rust
                 org-link-kind org-link-kind-rust
                 org-link-target-key org-link-target-key-rust
                 org-link-protocol org-link-protocol-rust
                 org-link-protocol-path org-link-protocol-path-rust
                 org-link-file-path org-link-file-path-rust
                 org-link-attachment-path org-link-attachment-path-rust
                 org-link-search org-link-search-rust
                 org-link-file-path-kind org-link-file-path-kind-rust
                 org-link-search-kind org-link-search-kind-rust
                 org-link-search-value org-link-search-value-rust
                 org-expand-link-abbreviation
                 org-expand-link-abbreviation-rust)
        (only-in "citation-functions.ss"
                 citation-style citation-style-rust
                 citation-variant citation-variant-rust)
        (only-in "document-keyword-properties.ss"
                 keyword-word keyword-word-rust
                 keyword-words keyword-words-rust
                 keyword-tag-words keyword-tag-words-rust
                 keyword-first-word keyword-first-word-rust
                 keyword-rest keyword-rest-rust
                 keyword-option-value keyword-option-value-rust
                 keyword-option-present? keyword-option-present-rust)
        (only-in "table-properties.ss"
                 table-column-cookie-match? table-column-cookie-match-rust
                 table-column-cookie-kind table-column-cookie-kind-rust)
        (only-in "affiliated-properties.ss"
                 org-affiliated-keyword? org-affiliated-keyword-rust)
        (only-in "catalog.ss" +org-affiliated-keywords+)
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
             => #t)
      (check (org-test-source-structured?
              "languages/org/v1/modules/org-elements/affiliated-properties.ss")
             => #t))
    (test-case "affiliated keyword membership is Scheme-owned and AOT projected"
      (check-org-headline-ir
       org-affiliated-keyword-rust 'org_affiliated_keyword_p
       "languages/org/v1/modules/org-elements/generated/org_affiliated_keyword_p.ir.json")
      (check (org-affiliated-keyword? "name" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "NAME" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "ATTR_HTML" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "attr_latex" +org-affiliated-keywords+) => #t)
      (check (org-affiliated-keyword? "TODO" +org-affiliated-keywords+) => #f))
    (test-case "link kind is Scheme-owned and AOT projected"
      (check-org-headline-ir
       org-image-link-rust 'org_image_link_p
       "languages/org/v1/modules/org-elements/generated/org_image_link_p.ir.json")
      (check (org-image-link? "diagram.svg") => #t)
      (check (org-image-link? "diagram.svg?size=2") => #f)
      (check (org-image-link? "https://example.test/doc.org") => #f)
      (check-org-headline-ir
       org-link-kind-rust 'org_link_kind
       "languages/org/v1/modules/org-elements/generated/org_link_kind.ir.json")
      (check-org-headline-ir
       org-link-protocol-rust 'org_link_protocol
       "languages/org/v1/modules/org-elements/generated/org_link_protocol.ir.json")
      (check-org-headline-ir
       org-link-protocol-path-rust 'org_link_protocol_path
       "languages/org/v1/modules/org-elements/generated/org_link_protocol_path.ir.json")
      (check-org-headline-ir
       org-link-target-key-rust 'org_link_target_key
       "languages/org/v1/modules/org-elements/generated/org_link_target_key.ir.json")
      (check-org-headline-ir
       org-link-file-path-rust 'org_link_file_path
       "languages/org/v1/modules/org-elements/generated/org_link_file_path.ir.json")
      (check-org-headline-ir
       org-link-attachment-path-rust 'org_link_attachment_path
       "languages/org/v1/modules/org-elements/generated/org_link_attachment_path.ir.json")
      (check-org-headline-ir
       org-link-search-rust 'org_link_search
       "languages/org/v1/modules/org-elements/generated/org_link_search.ir.json")
      (check-org-headline-ir
       org-link-file-path-kind-rust 'org_link_file_path_kind
       "languages/org/v1/modules/org-elements/generated/org_link_file_path_kind.ir.json")
      (check-org-headline-ir
       org-link-search-kind-rust 'org_link_search_kind
       "languages/org/v1/modules/org-elements/generated/org_link_search_kind.ir.json")
      (check-org-headline-ir
       org-link-search-value-rust 'org_link_search_value
       "languages/org/v1/modules/org-elements/generated/org_link_search_value.ir.json")
      (check-org-headline-ir
       org-expand-link-abbreviation-rust 'org_expand_link_abbreviation
       "languages/org/v1/modules/org-elements/generated/org_expand_link_abbreviation.ir.json")
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
    (test-case "document keyword values use Scheme-owned AOT algorithms"
      (check-org-headline-ir keyword-word-rust 'keyword_word
       "languages/org/v1/modules/org-elements/generated/keyword_word.ir.json")
      (check-org-headline-ir keyword-words-rust 'keyword_words
       "languages/org/v1/modules/org-elements/generated/keyword_words.ir.json")
      (check-org-headline-ir keyword-tag-words-rust 'keyword_tag_words
       "languages/org/v1/modules/org-elements/generated/keyword_tag_words.ir.json")
      (check-org-headline-ir keyword-first-word-rust 'keyword_first_word
       "languages/org/v1/modules/org-elements/generated/keyword_first_word.ir.json")
      (check-org-headline-ir keyword-rest-rust 'keyword_rest
       "languages/org/v1/modules/org-elements/generated/keyword_rest.ir.json")
      (check-org-headline-ir keyword-option-value-rust 'keyword_option_value
       "languages/org/v1/modules/org-elements/generated/keyword_option_value.ir.json")
      (check-org-headline-ir keyword-option-present-rust 'keyword_option_present_p
       "languages/org/v1/modules/org-elements/generated/keyword_option_present_p.ir.json")
      (check (keyword-word "  evidence ") => "evidence")
      (check (keyword-words " alpha  beta\tgamma ")
             => '("alpha" "beta" "gamma"))
      (check (keyword-tag-words " :alpha:beta:  gamma : ")
             => '("alpha" "beta" "gamma"))
      (check (keyword-first-word "  name   value with spaces ") => "name")
      (check (keyword-rest "  name   value with spaces ")
             => "value with spaces")
      (check (keyword-option-value "H:2 -:nil e:t H:3" "H") => "3")
      (check (keyword-option-value "H:2 H" "H") => "2")
      (check (keyword-option-value "H:2 -:nil" "missing") => "")
      (check (keyword-option-present? "H:2 H:" "H") => #t)
      (check (keyword-option-present? "H -:nil" "H") => #f))
    (test-case "table cookie classification is Scheme-owned and AOT projected"
      (check-org-headline-ir table-column-cookie-match-rust
       'table_column_cookie_match_p
       "languages/org/v1/modules/org-elements/generated/table_column_cookie_match_p.ir.json")
      (check-org-headline-ir table-column-cookie-kind-rust
       'table_column_cookie_kind
       "languages/org/v1/modules/org-elements/generated/table_column_cookie_kind.ir.json")
      (check (table-column-cookie-match? "r3" "r") => #t)
      (check (table-column-cookie-match? "r-1" "r") => #f)
      (check (table-column-cookie-kind " <l> ") => "left")
      (check (table-column-cookie-kind "<c12>") => "center")
      (check (table-column-cookie-kind "<r3>") => "right")
      (check (table-column-cookie-kind "<10>") => "width")
      (check (table-column-cookie-kind "<l>junk>") => "")
      (check (table-column-cookie-kind "<r-1>") => "")
      (check (table-column-cookie-kind "Text") => ""))
    (test-case "citation header style is Scheme-owned and AOT projected"
      (check-org-headline-ir
       citation-style-rust 'citation_style
       "languages/org/v1/modules/org-elements/generated/citation_style.ir.json")
      (check-org-headline-ir
       citation-variant-rust 'citation_variant
       "languages/org/v1/modules/org-elements/generated/citation_variant.ir.json")
      (check (citation-style "[cite:") => "nil")
      (check (citation-style "[cite/text:") => "text")
      (check (citation-style "[cite/noauthor/bare:") => "noauthor")
      (check (citation-variant "[cite:") => "")
      (check (citation-variant "[cite/noauthor/bare:") => "bare"))
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
       todo-word-name-rust 'todo_word_name
       "languages/org/v1/modules/org-elements/generated/todo_word_name.ir.json")
      (check-org-headline-ir
       todo-open-words-rust 'todo_open_words
       "languages/org/v1/modules/org-elements/generated/todo_open_words.ir.json")
      (check-org-headline-ir
       todo-done-words-rust 'todo_done_words
       "languages/org/v1/modules/org-elements/generated/todo_done_words.ir.json")
      (check (todo-word-name "WAIT(w@/!)") => "WAIT")
      (check (todo-open-words "NEXT(n) WAIT(w@/!) | DONE (d)")
             => '("NEXT" "WAIT"))
      (check (todo-done-words "NEXT(n) WAIT(w@/!) | DONE (d)")
             => '("DONE" ""))
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
       headline-source-title-rust 'headline_source_title
       "languages/org/v1/modules/org-elements/generated/headline_source_title.ir.json")
      (check (headline-source-title "TODO [#A] *Inline* task " "TODO")
             => "*Inline* task ")
      (check (headline-source-title "  plain  " "") => "plain  ")
      (check-org-headline-ir
       planning-key-kind-rust 'planning_key_kind
       "languages/org/v1/modules/org-elements/generated/planning_key_kind.ir.json")
      (check (planning-key-kind "scheduled") => "scheduled")
      (check (planning-key-kind "CLOSED") => "closed")
      (check-org-headline-ir
       priority-token-rust 'priority_token_p
       "languages/org/v1/modules/org-elements/generated/priority_token_p.ir.json")
      (check-org-headline-ir
       headline-display-title-rust 'headline_display_title
       "languages/org/v1/modules/org-elements/generated/headline_display_title.ir.json")
      (check-org-headline-ir
       headline-anchor-slug-rust 'headline_anchor_slug
       "languages/org/v1/modules/org-elements/generated/headline_anchor_slug.ir.json")
      (check (headline-anchor-slug "  Mixed  Case  ") => "mixed-case")
      (check (headline-anchor-slug "RÉSUMÉ Notes") => "résumé-notes")
      (check-org-headline-ir
       headline-comment-rust 'headline_comment_p
       "languages/org/v1/modules/org-elements/generated/headline_comment_p.ir.json")
      (check (headline-comment? "COMMENT Hidden") => #t)
      (check (headline-comment? "COMMENT") => #t)
      (check (headline-comment? "comment Visible") => #f)
      (check (headline-comment? "COMMENTARY Visible") => #f)
      (check-org-headline-ir
       memory-headline-state-rust 'memory_headline_state
       "languages/org/v1/modules/org-elements/generated/memory_headline_state.ir.json")
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
      (check-org-headline-ir
       headline-priority-cookie-rust 'headline_priority_cookie
       "languages/org/v1/modules/org-elements/generated/headline_priority_cookie.ir.json")
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
