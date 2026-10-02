#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Emit Org table metadata IR from its Scheme source.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "table-properties.ss"
                 table-column-cookie-match-rust
                 table-column-cookie-kind-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-table-ir.ss OUTPUT_DIR"))
(def output-dir (car (reverse arguments)))
(write-rust-function-ir
 (path-expand "table_column_cookie_match_p.ir.json" output-dir)
 table-column-cookie-match-rust)
(write-rust-function-ir
 (path-expand "table_column_cookie_kind.ir.json" output-dir)
 table-column-cookie-kind-rust)
