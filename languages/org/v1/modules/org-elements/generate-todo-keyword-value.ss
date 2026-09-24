#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Compile the Scheme TODO keyword value to a Cargo-only Rust function.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-syntax)
        (only-in "headline-properties.ss"
                 todo-keyword-from-directives-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-todo-keyword-value.ss OUTPUT.rs"))

(write-rust-syntax
 (car (reverse arguments)) todo-keyword-from-directives-rust)
