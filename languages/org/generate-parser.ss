#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only AOT entry. Cargo consumes the committed Rust product.

(import (only-in :gerbil-parser/rust-runtime-support
                 generate-language-rust-runtime-module)
        (only-in "grammar.ss" org-mode-language-grammar))

(export main)

(def (main output-path)
  (generate-language-rust-runtime-module output-path org-mode-language-grammar))
