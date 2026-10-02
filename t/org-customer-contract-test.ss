;;; -*- Gerbil -*-
;;; Consumer Contract source and committed Rust artifact are one AOT product.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/misc/ports read-all-as-string)
        (only-in "../languages/org/v1/modules/org-contract/aot.ss"
                 org-contract-rust-source)
        (only-in "../languages/org/v1/modules/org-contract/interface.ss"
                 org-contract-definition-id)
        (only-in "../tests/fixtures/org-contract/generated/customer-contract-source.ss"
                 org-contract-definitions))
(export customer-contract-test)

(def customer-contract-test
  (test-suite "consumer Org Contract AOT"
    (test-case "consumer contract pack matches admitted POO declarations"
      (check (call-with-input-file
              "tests/fixtures/org-contract/generated/customer-contract-pack.rs"
              read-all-as-string)
             => (org-contract-rust-source org-contract-definitions)))
    (test-case "consumer definitions stay POO-owned"
      (check (map org-contract-definition-id org-contract-definitions)
             => '("customer.review-evidence" "customer.has-task")))))
