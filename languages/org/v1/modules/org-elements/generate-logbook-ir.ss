#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; AOT projection of Scheme-owned LOGBOOK line classification.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "logbook-properties.ss" logbook-line-kind-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-logbook-ir.ss OUTPUT_DIR"))

(def output-dir (car (reverse arguments)))
(write-rust-function-ir
 (path-expand "logbook_line_kind.ir.json" output-dir)
 logbook-line-kind-rust)
