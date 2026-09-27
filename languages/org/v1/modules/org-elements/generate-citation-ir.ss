#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Project Scheme citation functions to typed Rust function IR.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "citation-functions.ss"
                 citation-style-rust citation-variant-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-citation-ir.ss OUTPUT_DIR"))

(def output-dir (car (reverse arguments)))
(for-each
 (lambda (entry)
   (write-rust-function-ir
    (path-expand (car entry) output-dir)
    (cdr entry)))
 (list (cons "citation_style.ir.json" citation-style-rust)
       (cons "citation_variant.ir.json" citation-variant-rust)))
