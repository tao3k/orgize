;;; -*- Gerbil -*-
;;; Consumer-authored query source and Rust artifact remain one AOT product.

(import (only-in :std/test check test-case test-suite)
        (only-in :std/misc/ports read-all-as-string)
        (only-in "../../../languages/org/v1/modules/org-elements/aot.ss"
                 org-element-query-rust-source)
        (only-in "generated/customer-query-source.ss" org-element-queries))
(export customer-query-test)

(def customer-query-test
  (test-suite "consumer Org Elements AOT"
    (test-case "consumer query pack matches its Scheme POO source"
      (check (call-with-input-file
              "tests/fixtures/org-elements/generated/customer-query-pack.rs"
              read-all-as-string)
             => (org-element-query-rust-source org-element-queries)))))
