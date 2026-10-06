#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Development-only AOT entry. Cargo consumes the committed Rust product.

(import (only-in :gerbil-parser/rust-rowan-support
                 generate-language-rust-rowan-module)
        (only-in "grammar.ss" org-v1-language-grammar))

(export main)

(def (main output-path)
  (generate-language-rust-rowan-module output-path org-v1-language-grammar))
