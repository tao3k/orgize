#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only AOT entry. Cargo consumes the committed Rust product.

(import (only-in :gerbil-parser/rust-rowan-support
                 generate-language-rust-rowan-module)
        (only-in "grammar.ss" org-v1-language-grammar))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-parser.ss OUTPUT.rs"))

(generate-language-rust-rowan-module
 (car (reverse arguments))
 org-v1-language-grammar)
