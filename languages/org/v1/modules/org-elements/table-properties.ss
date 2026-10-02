;;; -*- Gerbil -*-
;;; Org table metadata predicates compiled from Scheme into Rust.

(import (only-in :std/string/misc string-trim)
        (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-after string-before string-prefix?
                 string-suffix? string-unsigned-at-most?))
(export table-column-cookie-match? table-column-cookie-match-rust
        table-column-cookie-kind table-column-cookie-kind-rust)

;; A width cookie is bounded by Rust's signed integer range for both Scheme
;; and generated Rust. Larger widths are not meaningful for a table display.
(define-rust-pure table-column-cookie-match? table-column-cookie-match-rust
  ((inner "&str") (prefix "&str")) "bool"
  (or (equal? inner prefix)
      (and (string-prefix? inner prefix)
           (string-unsigned-at-most?
            (string-after inner prefix) 9223372036854775807))))

(define-rust-pure table-column-cookie-kind table-column-cookie-kind-rust
  ((cell "&str")) "&'static str"
  (using ((table-column-cookie-match? "&str" "&str"))
    (let* ((text (string-trim cell))
           (after-open (string-after text "<"))
           (inner (string-before after-open ">"))
           (width-cookie? (string-unsigned-at-most?
                           inner 9223372036854775807)))
      (if (and (string-prefix? text "<")
               (string-suffix? text ">")
               (equal? (string-after text ">") ""))
        (if width-cookie?
          "width"
          (if (table-column-cookie-match? inner "l") "left"
            (if (table-column-cookie-match? inner "c") "center"
              (if (table-column-cookie-match? inner "r") "right" ""))))
        ""))))
