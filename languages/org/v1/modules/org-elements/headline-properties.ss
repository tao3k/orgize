;;; -*- Gerbil -*-
;;; Org-only headline properties layered onto the existing Element graph.
;;; No second headline node or parser-engine semantics are introduced.

(import (only-in :std/string/misc string-trim)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-before ascii-ci=?
                 string-after string-first-word
                 string-rest-after-first-word string-last-word
                 string-before-last-word string-prefix? string-suffix?
                 string-words string-single-ascii-uppercase?
                 string-unsigned-at-most?)
        (only-in "types.ss" org-element-graph-view?)
        (only-in "objects.ss"
                 make-org-element-graph-view
                 make-org-headline-properties org-headline-property-field
                 org-element-graph-records org-element-graph-id-of
                 org-element-graph-parent-of org-element-graph-kind-of
                 org-element-graph-field-of))
(export org-element-with-headline-properties
        todo-directive-rust
        todo-word-name todo-word-name-rust
        todo-open-words todo-open-words-rust
        todo-done-words todo-done-words-rust
        todo-state-from-directives todo-state-from-directives-rust
        todo-keyword-from-directives todo-keyword-from-directives-rust
        headline-content-after-todo headline-content-after-todo-rust
        headline-display-title headline-display-title-rust
        headline-comment? headline-comment-rust
        priority-token? priority-token-rust
        todo-keyword-matches? todo-keyword-matches-rust
        memory-headline-state memory-headline-state-rust)

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

;; A file-local directive's keyword words are an Org-owned algorithm.  The
;; AOT functions return typed vectors, not generated source or a string wire.
(define-rust-pure todo-word-name todo-word-name-rust
  ((word "&str")) "String"
  (string-before word "("))

(define-rust-pure todo-open-words todo-open-words-rust
  ((directive "&str")) "Vec<String>"
  (using ((todo-word-name "&str"))
    (map todo-word-name
         (string-words (string-before directive "|")))))

(define-rust-pure todo-done-words todo-done-words-rust
  ((directive "&str")) "Vec<String>"
  (using ((todo-word-name "&str"))
    (map todo-word-name
         (string-words (string-after directive "|")))))

;; File-local keyword Elements override the caller's configured TODO profile.
;; The same executable Scheme function is lowered to Rust.
(define-rust-pure todo-state-from-directives todo-state-from-directives-rust
  ((title "&str") (directives "&[String]")
   (configured-todo "&[String]") (configured-done "&[String]")) "&'static str"
  (let* ((candidate (string-first-word title)))
    (if (equal? candidate "") ""
      (if (null? directives)
        (if (ormap (lambda (word) (equal? candidate word)) configured-todo)
          "todo"
          (if (ormap (lambda (word) (equal? candidate word)) configured-done)
            "done" ""))
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
  ((title "&str") (directives "&[String]")
   (configured-todo "&[String]") (configured-done "&[String]")) "String"
  (using ((todo-state-from-directives "&str" "&[String]"
                                      "&[String]" "&[String]"))
    (if (equal? (todo-state-from-directives
                 title directives configured-todo configured-done) "")
      ""
      (string-first-word title))))

(define-rust-pure headline-content-after-todo headline-content-after-todo-rust
  ((title "&str") (directives "&[String]")
   (configured-todo "&[String]") (configured-done "&[String]")) "String"
  (using ((todo-keyword-from-directives "&str" "&[String]"
                                        "&[String]" "&[String]"))
    (if (equal? (todo-keyword-from-directives
                 title directives configured-todo configured-done) "")
      (string-trim title)
      (string-rest-after-first-word title))))

(define-rust-pure priority-token? priority-token-rust
  ((word "&str")) "bool"
  (let* ((after-prefix (string-after word "[#"))
         (inner (string-before after-prefix "]")))
    (and (string-prefix? word "[#")
         (string-suffix? word "]")
         (equal? (string-after after-prefix "]") "")
         (or (string-single-ascii-uppercase? inner)
             (string-unsigned-at-most? inner 64)))))

(define-rust-pure headline-display-title headline-display-title-rust
  ((content "&str") (has-tags "bool")) "String"
  (using ((priority-token? "&str"))
    (let* ((trimmed (string-trim content))
           (first (string-first-word trimmed))
           (without-priority
            (if (priority-token? first)
              (string-rest-after-first-word trimmed)
              trimmed)))
      (if has-tags
        (string-before-last-word without-priority)
        (string-trim without-priority)))))

;; Org's COMMENT marker is a case-sensitive headline word after TODO and
;; priority have been resolved. Structural keywords remain case-insensitive.
(define-rust-pure headline-comment? headline-comment-rust
  ((display-title "&str")) "bool"
  (equal? (string-first-word display-title) "COMMENT"))

;; A query checks the source keyword only after the same Scheme-owned state
;; algorithm admits it under file-local or configured declarations. The call is
;; lowered to Rust with an explicit typed pure-function signature.
(define-rust-pure todo-keyword-matches? todo-keyword-matches-rust
  ((title "&str") (directives "&[String]")
   (configured-todo "&[String]") (configured-done "&[String]")
   (expected "&str")) "bool"
  (using ((todo-state-from-directives "&str" "&[String]"
                                      "&[String]" "&[String]"))
    (let* ((state (todo-state-from-directives
                   title directives configured-todo configured-done))
           (candidate (string-first-word title)))
      (and (or (equal? state "todo") (equal? state "done"))
           (equal? candidate expected)))))

;; Memory is a projection of admitted headline, planning and tag Elements.
;; Rust supplies those typed Element facts; this Scheme function owns their
;; lifecycle meaning in both the interpreter and the AOT consumer.
(define-rust-pure memory-headline-state memory-headline-state-rust
  ((todo-type "&str") (closed "bool") (planned "bool")
   (archived "bool")) "&'static str"
  (if archived "archived"
    (if (or (equal? todo-type "done") closed) "closed"
      (if (or (equal? todo-type "todo") planned) "current"
        "background"))))

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

(def (decode-headline title directives configured-todo configured-done)
  (let* ((state-value (todo-state-from-directives
                       title directives configured-todo configured-done))
         (state (if (equal? state-value "") #f state-value))
         (todo-value (todo-keyword-from-directives
                      title directives configured-todo configured-done))
         (todo (and state (not (equal? todo-value "")) todo-value))
         (after-todo (headline-content-after-todo
                      title directives configured-todo configured-done))
         (next (split-first after-todo))
         (priority (and (priority-token? (car next))
                        (substring (car next) 2
                                   (- (string-length (car next)) 1))))
         (after-priority (if priority (cdr next) after-todo))
         (last (or (split-last after-priority)
                   (and (tag-token after-priority)
                        (cons "" after-priority))))
         (tags (and last (tag-token (cdr last)))))
    (make-org-headline-properties
     title (if tags (car last) (string-trim after-priority))
     todo state priority (or tags '()))))

(def (org-element-with-headline-properties
      graph (configured-todo '("TODO")) (configured-done '("DONE")))
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
       (when (member (kind-of record) '("headline" "inlinetask"))
         (let (title (field-of record "title"))
           (when (string? title)
             (hash-put! headlines (id-of record)
                        (decode-headline title directives
                                         configured-todo configured-done))))))
     records)
    (make-org-element-graph-view
     records id-of parent-of kind-of
     (lambda (record name)
       (let (properties (hash-get headlines (id-of record)))
         (if properties
           (let-values (((known? projected)
                         (org-headline-property-field properties name)))
             (if known? projected (field-of record name)))
           (field-of record name)))))))
