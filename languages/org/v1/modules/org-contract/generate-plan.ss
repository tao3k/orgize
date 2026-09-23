#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Scheme/POO contract -> Rust AOT plan; Cargo consumes committed output.

(import (only-in "aot.ss" generate-org-contract-rust-module)
        (only-in "generated/contract-source.ss" org-contract-definitions))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-plan.ss OUTPUT.rs"))

(generate-org-contract-rust-module
 (car (reverse arguments)) org-contract-definitions)
