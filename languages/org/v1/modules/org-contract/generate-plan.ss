#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Scheme/POO contract -> Rust AOT plan; Cargo consumes committed output.

(import (only-in "aot.ss" generate-org-contract-rust-module))

(export main)

;; The selected source was tangled from a tagged Org block and imports only
;; the public POO feature interfaces. No Scheme runtime is needed by Cargo.
(def (main output-path
           (source-path "languages/org/v1/modules/org-contract/generated/contract-source.ss"))
  (load source-path)
  (generate-org-contract-rust-module output-path org-contract-definitions))
