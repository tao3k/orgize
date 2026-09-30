#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Rebuild the consumer Contract pack from its admitted Scheme POO source.

(import (only-in "../../../languages/org/v1/modules/org-contract/aot.ss"
                 generate-org-contract-rust-module)
        (only-in "generated/customer-contract-source.ss"
                 org-contract-definitions))

(def arguments (command-line))
(unless (> (length arguments) 2)
  (error "usage: generate-customer-contracts.ss OUTPUT.rs"))

(generate-org-contract-rust-module
 (car (reverse arguments)) org-contract-definitions)
