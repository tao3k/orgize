#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Project the Scheme-owned headline algorithms to typed Rust function IR.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "headline-properties.ss"
                 todo-directive-rust
                 todo-word-name-rust todo-open-words-rust
                 todo-done-words-rust
                 todo-state-from-directives-rust
                 todo-keyword-matches-rust
                 todo-keyword-from-directives-rust
                 headline-content-after-todo-rust
                 headline-source-title-rust
                 planning-key-kind-rust
                 headline-display-title-rust headline-anchor-slug-rust
                 priority-token-rust
                 headline-priority-cookie-rust
                 memory-headline-state-rust headline-comment-rust))

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
       (cons "todo_word_name.ir.json" todo-word-name-rust)
       (cons "todo_open_words.ir.json" todo-open-words-rust)
       (cons "todo_done_words.ir.json" todo-done-words-rust)
       (cons "todo_state_from_directives.ir.json" todo-state-from-directives-rust)
       (cons "todo_keyword_matches_p.ir.json" todo-keyword-matches-rust)
       (cons "todo_keyword_from_directives.ir.json" todo-keyword-from-directives-rust)
       (cons "headline_content_after_todo.ir.json" headline-content-after-todo-rust)
       (cons "headline_source_title.ir.json" headline-source-title-rust)
       (cons "planning_key_kind.ir.json" planning-key-kind-rust)
       (cons "priority_token_p.ir.json" priority-token-rust)
       (cons "headline_priority_cookie.ir.json" headline-priority-cookie-rust)
       (cons "headline_display_title.ir.json" headline-display-title-rust)
       (cons "headline_anchor_slug.ir.json" headline-anchor-slug-rust)
       (cons "headline_comment_p.ir.json" headline-comment-rust)
       (cons "memory_headline_state.ir.json" memory-headline-state-rust)))
