#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Project the Scheme-owned headline algorithms to typed Rust function IR.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "headline-properties.ss"
                 todo-directive-rust
                 todo-state-from-directives-rust
                 todo-keyword-matches-rust
                 todo-keyword-from-directives-rust
                 headline-content-after-todo-rust
                 headline-display-title-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-headline-ir.ss OUTPUT_DIR"))

(def output-dir (car (reverse arguments)))
(for-each
 (lambda (entry)
   (write-rust-function-ir
    (path-expand (car entry) output-dir)
    (cdr entry)))
 (list (cons "todo_directive_p.ir.json" todo-directive-rust)
       (cons "todo_state_from_directives.ir.json" todo-state-from-directives-rust)
       (cons "todo_keyword_matches_p.ir.json" todo-keyword-matches-rust)
       (cons "todo_keyword_from_directives.ir.json" todo-keyword-from-directives-rust)
       (cons "headline_content_after_todo.ir.json" headline-content-after-todo-rust)
       (cons "headline_display_title.ir.json" headline-display-title-rust)))
