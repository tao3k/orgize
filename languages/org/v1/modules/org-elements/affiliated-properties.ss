;;; -*- Gerbil -*-
;;; Org Elements owns affiliated keyword membership in Scheme and AOT Rust.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure ascii-ci=? string-before))
(export org-affiliated-keyword? org-affiliated-keyword-rust)

(define-rust-pure org-affiliated-keyword? org-affiliated-keyword-rust
  ((key "&str") (names "&[String]")) "bool"
  (or (ascii-ci=? (string-before key "_") "ATTR")
      (ormap (lambda (name) (ascii-ci=? key name)) names)))
