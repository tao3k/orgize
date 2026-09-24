;;; -*- Gerbil -*-
;;; Org-only headline properties layered onto the existing Element graph.
;;; No second headline node or parser-engine semantics are introduced.

(import (only-in :std/string/misc string-trim)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-before ascii-ci=?
                 string-after string-first-word string-words)
        (only-in "types.ss" org-element-graph-view?)
        (only-in "objects.ss"
                 make-org-element-graph-view
                 org-element-graph-records org-element-graph-id-of
                 org-element-graph-parent-of org-element-graph-kind-of
                 org-element-graph-field-of))
(export org-element-with-headline-properties
        todo-directive-rust
        todo-state-from-directives todo-state-from-directives-rust
        todo-keyword-from-directives todo-keyword-from-directives-rust
        todo-keyword-matches? todo-keyword-matches-rust)

(defstruct headline-properties (source-title title todo-keyword todo-type
                                          priority tags))

(def (split-first value)
  (let* ((text (string-trim value)) (size (string-length text)))
    (let loop ((index 0))
      (cond
       ((= index size) (cons text ""))
       ((char-whitespace? (string-ref text index))
        (cons (substring text 0 index)
              (string-trim (substring text index size))))
       (else (loop (+ index 1)))))))

(def (split-last value)
  (let (size (string-length value))
    (let loop ((index (- size 1)))
      (cond
       ((< index 0) #f)
       ((char-whitespace? (string-ref value index))
        (cons (string-trim (substring value 0 index))
              (string-trim (substring value (+ index 1) size))))
       (else (loop (- index 1)))))))

(define-rust-pure todo-directive? todo-directive-rust
  ((key "&str")) "bool"
  (or (ascii-ci=? key "TODO")
      (ascii-ci=? key "SEQ_TODO")
      (ascii-ci=? key "TYP_TODO")))

;; File-local keyword Elements are the authority. The same executable Scheme
;; function is lowered to Rust; callers never supply a separate TODO profile.
(define-rust-pure todo-state-from-directives todo-state-from-directives-rust
  ((title "&str") (directives "&[String]")) "&'static str"
  (let* ((candidate (string-first-word title)))
    (if (equal? candidate "") ""
      (if (null? directives)
        (if (equal? candidate "TODO") "todo"
          (if (equal? candidate "DONE") "done" ""))
        (if (ormap
             (lambda (directive)
               (let* ((open-side (string-before directive "|")))
                 (ormap
                  (lambda (word)
                    (equal? candidate (string-before word "(")))
                  (string-words open-side))))
             directives)
          "todo"
          (if (ormap
               (lambda (directive)
                 (let* ((done-side (string-after directive "|")))
                   (ormap
                    (lambda (word)
                      (equal? candidate (string-before word "(")))
                    (string-words done-side))))
               directives)
            "done" ""))))))

;; The keyword value is a source-owned string, not a Rust re-parse of title.
(define-rust-pure todo-keyword-from-directives todo-keyword-from-directives-rust
  ((title "&str") (directives "&[String]")) "String"
  (using ((todo-state-from-directives "&str" "&[String]"))
    (if (equal? (todo-state-from-directives title directives) "")
      ""
      (string-first-word title))))

;; A query checks the source keyword only after the same Scheme-owned state
;; algorithm admits it under file-local declarations. The dependency call is
;; lowered to Rust with an explicit typed pure-function signature.
(define-rust-pure todo-keyword-matches? todo-keyword-matches-rust
  ((title "&str") (directives "&[String]") (expected "&str")) "bool"
  (using ((todo-state-from-directives "&str" "&[String]"))
    (let* ((state (todo-state-from-directives title directives))
           (candidate (string-first-word title)))
      (and (or (equal? state "todo") (equal? state "done"))
           (equal? candidate expected)))))

(def (document-todo-directives records kind-of field-of)
  (let loop ((rest records) (directives '()))
    (if (null? rest)
      (reverse directives)
      (let* ((record (car rest))
             (key (and (equal? (kind-of record) "keyword")
                       (field-of record "key")))
             (value (and key (field-of record "value"))))
        (if (and (string? key) (todo-directive? key) (string? value))
          (loop (cdr rest) (cons value directives))
          (loop (cdr rest) directives))))))

(def (priority-token? word)
  (let (size (string-length word))
    (and (>= size 4)
         (char=? (string-ref word 0) #\[)
         (char=? (string-ref word 1) #\#)
         (char=? (string-ref word (- size 1)) #\])
         (let (inner (substring word 2 (- size 1)))
           (or (and (= (string-length inner) 1)
                    (char-alphabetic? (string-ref inner 0)))
               (let (number (string->number inner))
                 (and (integer? number) (<= 0 number 64))))))))

(def (tag-char? char)
  (or (char-alphabetic? char) (char-numeric? char)
      (memv char '(#\_ #\@ #\# #\%))))

(def (tag-token word)
  (let (size (string-length word))
    (and (> size 2)
         (char=? (string-ref word 0) #\:)
         (char=? (string-ref word (- size 1)) #\:)
         (let (tags (string-split (substring word 1 (- size 1)) #\:))
           (and (pair? tags)
                (andmap (lambda (tag)
                          (and (> (string-length tag) 0)
                               (let loop ((index 0))
                                 (or (= index (string-length tag))
                                     (and (tag-char? (string-ref tag index))
                                          (loop (+ index 1)))))))
                        tags)
                tags)))))

(def (decode-headline title directives)
  (let* ((first (split-first title))
         (state-value (todo-state-from-directives title directives))
         (state (if (equal? state-value "") #f state-value))
         (todo-value (todo-keyword-from-directives title directives))
         (todo (and state (not (equal? todo-value "")) todo-value))
         (after-todo (if todo (cdr first) (string-trim title)))
         (next (split-first after-todo))
         (priority (and (priority-token? (car next))
                        (substring (car next) 2
                                   (- (string-length (car next)) 1))))
         (after-priority (if priority (cdr next) after-todo))
         (last (or (split-last after-priority)
                   (and (tag-token after-priority)
                        (cons "" after-priority))))
         (tags (and last (tag-token (cdr last)))))
    (make-headline-properties
     title (if tags (car last) (string-trim after-priority))
     todo state priority (or tags '()))))

(def (org-element-with-headline-properties graph)
  (unless (org-element-graph-view? graph)
    (error "headline properties require an admitted Org Element graph" graph))
  (let* ((records (org-element-graph-records graph))
         (id-of (org-element-graph-id-of graph))
         (parent-of (org-element-graph-parent-of graph))
         (kind-of (org-element-graph-kind-of graph))
         (field-of (org-element-graph-field-of graph))
         (directives (document-todo-directives records kind-of field-of))
         (headlines (make-hash-table)))
    (for-each
     (lambda (record)
       (when (equal? (kind-of record) "headline")
         (let (title (field-of record "title"))
           (when (string? title)
             (hash-put! headlines (id-of record)
                        (decode-headline title directives))))))
     records)
    (make-org-element-graph-view
     records id-of parent-of kind-of
     (lambda (record name)
       (let (properties (hash-get headlines (id-of record)))
         (if properties
           (cond
            ((equal? name "title") (headline-properties-title properties))
            ((equal? name "source-title")
             (headline-properties-source-title properties))
            ((equal? name "raw-value")
             (headline-properties-title properties))
            ((equal? name "todo-keyword")
             (headline-properties-todo-keyword properties))
            ((equal? name "todo-type")
             (headline-properties-todo-type properties))
            ((equal? name "priority")
             (headline-properties-priority properties))
            ((equal? name "tags") (headline-properties-tags properties))
            (else (field-of record name)))
           (field-of record name)))))))
