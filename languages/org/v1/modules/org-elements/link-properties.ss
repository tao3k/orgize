;;; -*- Gerbil -*-
;;; Org Element link classification is Scheme-owned and lowered for Cargo.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-suffix?))
(export org-image-link? org-image-link-rust)

(define-rust-pure org-image-link? org-image-link-rust
  ((target "&str")) "bool"
  (or (string-suffix? target ".png")
      (string-suffix? target ".jpeg")
      (string-suffix? target ".jpg")
      (string-suffix? target ".gif")
      (string-suffix? target ".tiff")
      (string-suffix? target ".tif")
      (string-suffix? target ".xbm")
      (string-suffix? target ".xpm")
      (string-suffix? target ".pbm")
      (string-suffix? target ".pgm")
      (string-suffix? target ".ppm")
      (string-suffix? target ".webp")
      (string-suffix? target ".avif")
      (string-suffix? target ".svg")))
