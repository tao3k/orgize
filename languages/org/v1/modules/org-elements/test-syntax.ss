;;; -*- Gerbil -*-
;;; Domain-specific hygienic checks for Org Element catalog and graph queries.

(import (only-in :std/test check)
        (only-in :std/encoding/json JSONReadOptions string->json)
        (only-in :std/misc/ports read-all-as-string)
        (only-in :gerbil-parser/src/compiler/rust-syntax
                 rust-function-ir-json
                 rust-function-form? rust-function-form-name
                 rust-function-form-body rust-block-form-statements
                 rust-block-form-result rust-let-form? rust-let-form-value
                 rust-first-word-form? rust-if-form? rust-if-form-condition
                 rust-if-form-alternate rust-any-form? rust-empty-form?)
        (only-in :std/list/list every)
        (only-in "config.ss"
                 +org-element-kinds+ +org-greater-element-kinds+
                 +org-object-kinds+ +org-recursive-object-kinds+
                 +org-affiliated-keywords+ +org-object-restrictions+
                 +org-secondary-values+ org-object-allowed?
                 org-secondary-value?)
        (only-in "funs.ss" org-element-select org-element-property)
        (only-in "aot.ss" org-element-query-rust-source)
        (only-in "objects.ss"
                 org-named-element-query-id org-named-element-query-query))
(export check-org-element-catalog check-org-element-selection
        check-org-named-query-selection
        check-org-headline-properties
        check-org-headline-ir
        check-org-headline-state-aot
        check-org-element-query-aot
        org-test-form-structured? org-test-source-structured?
        org-test-sources)

;; The test contract inspects Scheme forms, not source substrings: quoted
;; negative fixtures and string literals cannot accidentally trip the gate.
(def (org-test-form-structured? form)
  (cond
   ((not (pair? form)) #t)
   ((eq? (car form) 'quote) #t)
   ((and (symbol? (car form))
         (memq (car form)
               '(display displayln string-append call-with-output-string)))
    #f)
   (else (and (org-test-form-structured? (car form))
              (org-test-form-structured? (cdr form))))))

(def (org-test-source-structured? path)
  (call-with-input-file path
    (lambda (port)
      (let loop ()
        (let (form (read port))
          (or (eof-object? form)
              (and (org-test-form-structured? form) (loop))))))))

(def (org-test-sources root)
  (apply append
         (map (lambda (name)
                (let* ((path (path-expand name root))
                       (info (file-info path)))
                  (cond
                   ((eq? (file-info-type info) 'directory)
                    (org-test-sources path))
                   ((string-suffix? "-test.ss" name) (list path))
                   (else '()))))
              (directory-files root))))

(def (catalog-unique? values)
  (let loop ((remaining values) (seen '()))
    (if (null? remaining) #t
        (and (not (member (car remaining) seen))
             (loop (cdr remaining) (cons (car remaining) seen))))))

(def (known-kind? kind)
  (if (or (equal? kind "org-data")
          (member kind +org-element-kinds+)
          (member kind +org-object-kinds+))
    #t #f))

(def (json-tree-equivalent? left right)
  (cond
   ((and (hash-table? left) (hash-table? right))
    (let ((keys (hash-keys left)) (other-keys (hash-keys right)))
      (and (= (length keys) (length other-keys))
           (let loop ((rest keys))
             (or (null? rest)
                 (and (member (car rest) other-keys)
                      (json-tree-equivalent?
                       (hash-get left (car rest))
                       (hash-get right (car rest)))
                      (loop (cdr rest))))))))
   ((and (list? left) (list? right))
    (and (= (length left) (length right))
         (let loop ((values left) (expected right))
           (or (null? values)
               (and (json-tree-equivalent? (car values) (car expected))
                    (loop (cdr values) (cdr expected)))))))
   (else (equal? left right))))

(defsyntax (check-org-element-catalog stx)
  (syntax-case stx ()
    ((_)
     (syntax
      (begin
        (check (every catalog-unique?
                      (list +org-element-kinds+
                            +org-greater-element-kinds+
                            +org-object-kinds+
                            +org-recursive-object-kinds+
                            +org-affiliated-keywords+
                            (map car +org-object-restrictions+))) => #t)
        (check (every known-kind? +org-greater-element-kinds+) => #t)
        (check (every (lambda (kind)
                        (if (member kind +org-object-kinds+) #t #f))
                      +org-recursive-object-kinds+) => #t)
        (check (every (lambda (entry)
                        (and (known-kind? (car entry))
                             (catalog-unique? (cdr entry))
                             (every (lambda (kind)
                                      (if (member kind +org-object-kinds+)
                                        #t #f))
                                    (cdr entry))))
                      +org-object-restrictions+) => #t)
        (check (every (lambda (entry)
                        (if (assoc (car entry) +org-object-restrictions+)
                          #t #f))
                      +org-secondary-values+) => #t)
        (check (org-object-allowed? "paragraph" "link") => #t)
        (check (org-object-allowed? "table-row" "table-cell") => #t)
        (check (equal? (cdr (assoc "table-row" +org-object-restrictions+))
                       '("table-cell")) => #t)
        (check (org-object-allowed? "table-row" "link") => #f)
        (check (org-object-allowed? "headline" "line-break") => #f)
        (check (org-object-allowed? "src-block" "bold") => #f)
        (check (org-secondary-value? "headline" "title") => #t)
        (check (org-secondary-value? "paragraph" "title") => #f))))))

(defsyntax (check-org-element-selection stx)
  (syntax-case stx ()
    ((_ query context scope targets id-of expected-ids)
     (syntax
      (check (map id-of (org-element-select query context scope targets))
             => expected-ids)))))

(defsyntax (check-org-named-query-selection stx)
  (syntax-case stx ()
    ((_ named context scope targets id-of expected-name expected-ids)
     (syntax
      (begin
        (check (org-named-element-query-id named) => expected-name)
        (check (map id-of
                    (org-element-select
                     (org-named-element-query-query named)
                     context scope targets))
               => expected-ids))))))

(defsyntax (check-org-element-query-aot stx)
  (syntax-case stx ()
    ((_ queries artifact)
     (syntax
      (check (call-with-input-file artifact read-all-as-string)
             => (org-element-query-rust-source queries))))))

(defsyntax (check-org-headline-properties stx)
  (syntax-case stx ()
    ((_ context record source title todo type priority tags)
     (syntax
      (begin
        (check (org-element-property context record "source-title") => source)
        (check (org-element-property context record "raw-value") => title)
        (check (org-element-property context record "title") => title)
        (check (org-element-property context record "todo-keyword") => todo)
        (check (org-element-property context record "todo-type") => type)
        (check (org-element-property context record "priority") => priority)
        (check (org-element-property context record "tags") => tags))))))

(defsyntax (check-org-headline-ir stx)
  (syntax-case stx ()
    ((_ function name fixture)
     (syntax
      (let* ((options (JSONReadOptions object-as-hash: #t))
             (ir (string->json (rust-function-ir-json function) options))
             (saved (call-with-input-file fixture
                      (lambda (port)
                        (string->json (read-all-as-string port) options)))))
        (check (rust-function-form? function) => #t)
        (check (rust-function-form-name function) => name)
        (check (hash-get ir "schema")
               => "gerbil-scheme-rust.rust-function-ir.v1")
        (check (hash-get ir "name") => (symbol->string name))
        (check (json-tree-equivalent? ir saved) => #t))))))

(defsyntax (check-org-headline-state-aot stx)
  (syntax-case stx ()
    ((_ function)
     (syntax
      (let* ((value function)
             (body (rust-function-form-body value))
             (bindings (rust-block-form-statements body))
             (branch (rust-block-form-result body))
             (declared (rust-if-form-alternate branch)))
        (check (rust-function-form? value) => #t)
        (check (rust-function-form-name value)
               => 'todo_state_from_directives)
        (check (length bindings) => 1)
        (check (rust-let-form? (car bindings)) => #t)
        (check (rust-first-word-form?
                (rust-let-form-value (car bindings))) => #t)
        (check (rust-if-form? branch) => #t)
        (check (rust-if-form? declared) => #t)
        (check (rust-empty-form? (rust-if-form-condition declared)) => #t)
        (check (rust-any-form?
                (rust-if-form-condition
                 (rust-if-form-alternate declared))) => #t))))))
