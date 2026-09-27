#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Project Org-owned link classification into typed Rust function IR.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "link-properties.ss"
                 org-image-link-rust org-link-kind-rust
                 org-link-target-key-rust
                 org-link-protocol-rust org-link-protocol-path-rust
                 org-link-file-path-rust org-link-search-rust
                 org-link-file-path-kind-rust org-link-search-kind-rust
                 org-link-search-value-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-link-ir.ss OUTPUT_DIR"))

(write-rust-function-ir
 (path-expand "org_image_link_p.ir.json" (car (reverse arguments)))
 org-image-link-rust)
(write-rust-function-ir
 (path-expand "org_link_kind.ir.json" (car (reverse arguments)))
 org-link-kind-rust)
(write-rust-function-ir
 (path-expand "org_link_target_key.ir.json" (car (reverse arguments)))
 org-link-target-key-rust)
(write-rust-function-ir
 (path-expand "org_link_protocol.ir.json" (car (reverse arguments)))
 org-link-protocol-rust)
(write-rust-function-ir
 (path-expand "org_link_protocol_path.ir.json" (car (reverse arguments)))
 org-link-protocol-path-rust)
(write-rust-function-ir
 (path-expand "org_link_file_path.ir.json" (car (reverse arguments)))
 org-link-file-path-rust)
(write-rust-function-ir
 (path-expand "org_link_search.ir.json" (car (reverse arguments)))
 org-link-search-rust)
(write-rust-function-ir
 (path-expand "org_link_file_path_kind.ir.json" (car (reverse arguments)))
 org-link-file-path-kind-rust)
(write-rust-function-ir
 (path-expand "org_link_search_kind.ir.json" (car (reverse arguments)))
 org-link-search-kind-rust)
(write-rust-function-ir
 (path-expand "org_link_search_value.ir.json" (car (reverse arguments)))
 org-link-search-value-rust)
