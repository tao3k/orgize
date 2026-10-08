#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Developer-only AOT projection; Cargo consumes the committed Rust module.

(import (only-in :gerbil-parser/rust-runtime-support
                 generate-language-rust-runtime-module)
        (only-in "grammar.ss" org-contract-language-grammar))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-parser.ss OUTPUT.rs"))

(generate-language-rust-runtime-module
 (car (reverse arguments))
 org-contract-language-grammar)
