;;; -*- Gerbil -*-
;;; Org citation style and variant remain Scheme-owned AOT algorithms.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-after string-before string-prefix?))
(export citation-style citation-style-rust
        citation-variant citation-variant-rust)

(define-rust-pure citation-style citation-style-rust
  ((head "&str")) "String"
  (let* ((after-cite (string-after head "[cite/"))
         (before-colon (string-before after-cite ":")))
    (if (string-prefix? head "[cite/")
      (string-before before-colon "/")
      "nil")))

(define-rust-pure citation-variant citation-variant-rust
  ((head "&str")) "&str"
  (let* ((after-cite (string-after head "[cite/"))
         (before-colon (string-before after-cite ":")))
    (if (string-prefix? head "[cite/")
      (string-after before-colon "/")
      "")))
