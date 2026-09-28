;;; -*- Gerbil -*-
;;; Org Element link classification is Scheme-owned and lowered for Cargo.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 define-rust-pure string-before string-after
                 string-replace
                 string-prefix? string-suffix?
                 string-unsigned-at-most?))
(export org-image-link? org-image-link-rust
        org-link-kind org-link-kind-rust
        org-link-target-key org-link-target-key-rust
        org-link-protocol org-link-protocol-rust
        org-link-protocol-path org-link-protocol-path-rust
        org-link-file-path org-link-file-path-rust
        org-link-attachment-path org-link-attachment-path-rust
        org-link-search org-link-search-rust
        org-link-file-path-kind org-link-file-path-kind-rust
        org-link-search-kind org-link-search-kind-rust
        org-link-search-value org-link-search-value-rust
        org-expand-link-abbreviation org-expand-link-abbreviation-rust)

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

;; The path syntax is classified once by Scheme. The Rust consumer may use the
;; projected category to look up a target, but must not reparse its prefix.
(define-rust-pure org-link-kind org-link-kind-rust
  ((path "&str")) "&'static str"
  (if (string-prefix? path "*") "headline"
    (if (string-prefix? path "#") "custom-id"
      (if (string-prefix? path "id:") "id"
        (if (string-prefix? path "fn:") "footnote"
          (if (string-prefix? path "coderef:") "code-ref"
            (if (equal? (string-before path ":") path)
              "fuzzy" "uri")))))))

(define-rust-pure org-link-protocol org-link-protocol-rust
  ((path "&str")) "&str"
  (string-before path ":"))

(define-rust-pure org-link-target-key org-link-target-key-rust
  ((path "&str")) "&str"
  (if (string-prefix? path "*")
    (string-after path "*")
    (if (string-prefix? path "id:")
      (string-before path "::")
      path)))

(define-rust-pure org-link-protocol-path org-link-protocol-path-rust
  ((path "&str")) "&str"
  (string-after path ":"))

(define-rust-pure org-link-file-path org-link-file-path-rust
  ((path "&str")) "&str"
  (let* ((after-protocol (string-after path "file:")))
    (string-before after-protocol "::")))

(define-rust-pure org-link-attachment-path org-link-attachment-path-rust
  ((path "&str")) "&str"
  (let* ((after-protocol (string-after path ":")))
    (string-before after-protocol "::")))

(define-rust-pure org-link-search org-link-search-rust
  ((path "&str")) "&str"
  (string-after path "::"))

(define-rust-pure org-link-file-path-kind org-link-file-path-kind-rust
  ((path "&str")) "&'static str"
  (if (equal? path "") "empty"
    (if (string-prefix? path "/ssh:") "remote"
      (if (string-prefix? path "/") "absolute"
        (if (string-prefix? path "~/") "home-relative"
          "relative")))))

(define-rust-pure org-link-search-kind org-link-search-kind-rust
  ((search "&str")) "&'static str"
  (let* ((line-number? (string-unsigned-at-most? search 4294967295)))
    (if (string-prefix? search "*") "headline"
      (if (string-prefix? search "#") "custom-id"
        (if (string-prefix? search "/") "regexp"
          (if line-number? "line-number" "text"))))))

(define-rust-pure org-link-search-value org-link-search-value-rust
  ((search "&str")) "&str"
  (if (string-prefix? search "*")
    (string-after search "*")
    (if (string-prefix? search "#")
      (string-after search "#")
      search)))

(define-rust-pure org-expand-link-abbreviation
  org-expand-link-abbreviation-rust
  ((replacement "&str") (path "&str") (encoded-path "&str")) "String"
  (if (and (equal? (string-before replacement "%s") replacement)
           (equal? (string-before replacement "%h") replacement))
    (string-join (list replacement path) "")
    (string-replace
     (string-replace replacement "%s" path)
     "%h" encoded-path)))
