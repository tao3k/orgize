#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; AOT projection of Scheme-owned LOGBOOK line classification.

(import (only-in :gerbil-parser/src/compiler/rust-syntax
                 write-rust-function-ir)
        (only-in "logbook-properties.ss"
                 logbook-content-line-rust logbook-line-kind-rust
                 logbook-state-quote-shape-rust
                 logbook-state-to-rust logbook-state-from-rust
                 logbook-clock-duration-shape-rust
                 logbook-clock-duration-value-rust))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-logbook-ir.ss OUTPUT_DIR"))

(def output-dir (car (reverse arguments)))
(write-rust-function-ir
 (path-expand "logbook_content_line.ir.json" output-dir)
 logbook-content-line-rust)
(write-rust-function-ir
 (path-expand "logbook_line_kind.ir.json" output-dir)
 logbook-line-kind-rust)
(write-rust-function-ir
 (path-expand "logbook_state_quote_shape.ir.json" output-dir)
 logbook-state-quote-shape-rust)
(write-rust-function-ir
 (path-expand "logbook_state_to.ir.json" output-dir)
 logbook-state-to-rust)
(write-rust-function-ir
 (path-expand "logbook_state_from.ir.json" output-dir)
 logbook-state-from-rust)
(write-rust-function-ir
 (path-expand "logbook_clock_duration_shape.ir.json" output-dir)
 logbook-clock-duration-shape-rust)
(write-rust-function-ir
 (path-expand "logbook_clock_duration_value.ir.json" output-dir)
 logbook-clock-duration-value-rust)
