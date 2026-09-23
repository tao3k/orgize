#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Thin developer AOT entry; Cargo consumers compile committed Rust tables.

(import (only-in :gerbil-parser/line-structure-support
                 generate-line-structure-rowan-module)
        (only-in "grammar.ss" org-v1-language-grammar)
        (only-in "parser.ss" org-v1-line-structure))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-structure.ss OUTPUT.rs"))

(generate-line-structure-rowan-module
 (car (reverse arguments))
 org-v1-language-grammar
 org-v1-line-structure)
