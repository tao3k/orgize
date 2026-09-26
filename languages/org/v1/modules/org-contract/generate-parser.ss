#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Developer-only AOT projection; Cargo consumes the committed Rust module.

(import (only-in :gerbil-parser/rust-rowan-support
                 generate-language-rust-rowan-module)
        (only-in "grammar.ss" org-contract-language-grammar))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-parser.ss OUTPUT.rs"))

(generate-language-rust-rowan-module
 (car (reverse arguments))
 org-contract-language-grammar)
