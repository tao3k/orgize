#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Scheme/POO contract -> Rust AOT plan; Cargo consumes committed output.

(import (only-in "aot.ss" generate-org-contract-rust-module))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-plan.ss OUTPUT.rs [CONTRACT_SOURCE.ss]"))

(def output-path (caddr arguments))
(def source-path
  (if (> (length arguments) 3)
    (cadddr arguments)
    "languages/org/v1/modules/org-contract/generated/contract-source.ss"))

;; The selected source was tangled from a tagged Org block and imports only
;; the public POO feature interfaces. No Scheme runtime is needed by Cargo.
(load source-path)

(generate-org-contract-rust-module
 output-path org-contract-definitions)
