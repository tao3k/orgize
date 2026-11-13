#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Compile tagged Org Element queries into a Cargo-only Rust query pack.

(import (only-in "aot.ss" generate-org-element-query-rust-module)
        (only-in "generated/query-source.ss" org-element-queries))

(export main)

(def (main output-path)
  (generate-org-element-query-rust-module output-path org-element-queries))
