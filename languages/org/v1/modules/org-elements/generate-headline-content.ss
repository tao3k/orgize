#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Compile the Scheme-owned TODO-stripped headline content to Rust.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "headline-properties.ss"
                 headline-content-after-todo-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-headline-content.ss OUTPUT.json"))

(write-rust-function-ir
 (car (reverse arguments)) headline-content-after-todo-rust)
