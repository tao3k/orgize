#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Compile the Scheme-owned affiliated keyword predicate to typed Rust IR.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "affiliated-properties.ss" org-affiliated-keyword-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-affiliated-ir.ss OUTPUT_DIR"))

(write-rust-function-ir
 (path-expand "org_affiliated_keyword_p.ir.json" (car (reverse arguments)))
 org-affiliated-keyword-rust)
