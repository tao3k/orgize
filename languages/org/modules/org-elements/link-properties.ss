;;; -*- Gerbil -*-
;;; Native Scheme link classification; Rust only admits projected values.

(import (only-in :gerbil-parser/src/compiler/rust-pure-aot
                 string-before string-after
                 string-replace
                 string-prefix? string-suffix?
                 string-unsigned-at-most?))
(export org-image-link?
        org-link-kind
        org-link-target-key
        org-link-protocol
        org-link-protocol-path
        org-link-file-path
        org-link-attachment-path
        org-link-search
        org-link-file-path-kind
        org-link-search-kind
        org-link-search-value
        org-expand-link-abbreviation)

(def (org-image-link? target)
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
(def (org-link-kind path)
  (if (string-prefix? path "*") "headline"
    (if (string-prefix? path "#") "custom-id"
      (if (string-prefix? path "id:") "id"
        (if (string-prefix? path "fn:") "footnote"
          (if (string-prefix? path "coderef:") "code-ref"
            (if (equal? (string-before path ":") path)
              "fuzzy" "uri")))))))

(def (org-link-protocol path)
  (string-before path ":"))

(def (org-link-target-key path)
  (if (string-prefix? path "*")
    (string-after path "*")
    (if (string-prefix? path "id:")
      (string-before path "::")
      path)))

(def (org-link-protocol-path path)
  (string-after path ":"))

(def (org-link-file-path path)
  (let* ((after-protocol (string-after path "file:")))
    (string-before after-protocol "::")))

(def (org-link-attachment-path path)
  (let* ((after-protocol (string-after path ":")))
    (string-before after-protocol "::")))

(def (org-link-search path)
  (string-after path "::"))

(def (org-link-file-path-kind path)
  (if (equal? path "") "empty"
    (if (string-prefix? path "/ssh:") "remote"
      (if (string-prefix? path "/") "absolute"
        (if (string-prefix? path "~/") "home-relative"
          "relative")))))

(def (org-link-search-kind search)
  (let* ((line-number? (string-unsigned-at-most? search 4294967295)))
    (if (string-prefix? search "*") "headline"
      (if (string-prefix? search "#") "custom-id"
        (if (string-prefix? search "/") "regexp"
          (if line-number? "line-number" "text"))))))

(def (org-link-search-value search)
  (if (string-prefix? search "*")
    (string-after search "*")
    (if (string-prefix? search "#")
      (string-after search "#")
      search)))

(def (org-expand-link-abbreviation replacement path encoded-path)
  (if (and (equal? (string-before replacement "%s") replacement)
           (equal? (string-before replacement "%h") replacement))
    (string-join (list replacement path) "")
    (string-replace
     (string-replace replacement "%s" path)
     "%h" encoded-path)))
