#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Compile tagged Org Element queries into a Cargo-only Rust query pack.

(import (only-in "aot.ss" generate-org-element-query-rust-module)
        (only-in "generated/query-source.ss" org-element-queries))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-queries.ss OUTPUT.rs"))

(generate-org-element-query-rust-module
 (car (reverse arguments)) org-element-queries)
