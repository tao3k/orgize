#!/usr/bin/env gxi
;;; -*- Gerbil -*-
;;; Rebuild the consumer Contract pack from its admitted Scheme POO source.

(import (only-in "../../../languages/org/v1/modules/org-contract/aot.ss"
                 generate-org-contract-rust-module)
        (only-in "generated/customer-contract-source.ss"
                 org-contract-definitions))

(export main)

(def (main output-path)
  (generate-org-contract-rust-module output-path org-contract-definitions))
