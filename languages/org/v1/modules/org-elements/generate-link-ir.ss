#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Project Org-owned link classification into typed Rust function IR.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "link-properties.ss" org-image-link-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-link-ir.ss OUTPUT_DIR"))

(write-rust-function-ir
 (path-expand "org_image_link_p.ir.json" (car (reverse arguments)))
 org-image-link-rust)
