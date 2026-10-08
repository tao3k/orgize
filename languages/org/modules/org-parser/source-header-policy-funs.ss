;;; -*- Gerbil -*-
;;; Pure Babel policy; Rust only constructs the admitted enum domains.
(import (only-in "property-token-funs.ss" org-ascii-lower org-property-tokens)
        (only-in "text-funs.ss" org-trim org-prefix-at?))
(export org-header-policy org-header-defaults org-header-language? org-alignment-cookie?)
(def (org-header-policy mode raw)
  (let (value (org-ascii-lower raw))
    (cond
     ((string=? mode "tangle") (if (member value '("yes" "no")) value "file"))
     ((string=? mode "mkdirp") (if (member value '("yes" "t")) "true" "false"))
     ((string=? mode "comments") (if (member value '("no" "link" "yes" "org" "both" "noweb")) value "other"))
     ((string=? mode "noweb")
      (let (tokens (map org-ascii-lower (org-property-tokens raw)))
        (cond ((member "strip-tangle" tokens) "strip")
              ((ormap (lambda (token) (member token '("yes" "tangle" "no-export" "strip-export"))) tokens) "expand")
              (else "disabled"))))
     ((string=? mode "result")
      (if (member value '("file" "list" "vector" "table" "scalar" "verbatim"
                         "raw" "html" "latex" "org" "code" "pp" "drawer" "link" "graphics"
                         "replace" "silent" "none" "discard" "append" "prepend" "output" "value")) value "other"))
     ((string=? mode "kind")
      (if (member value '("cache" "dir" "eval" "exports" "file" "file-desc" "file-ext"
                         "file-mode" "format" "hlines" "lint" "noweb" "output-dir" "results"
                         "runtime" "session" "tangle" "var")) value "other"))
     (else (error "invalid native header policy" mode)))))
(def (org-header-defaults kind)
  (unless (member kind '("block" "inline")) (error "invalid native header kind" kind))
  (let (inline? (string=? kind "inline"))
    (list '("eval" "yes") '("session" "none") '("results" "replace")
          (list "exports" (if inline? "results" "code")) '("cache" "no")
          '("noweb" "no") (list "hlines" (if inline? "yes" "no")) '("tangle" "no"))))
(def (org-header-language? key language)
  (let (key (org-ascii-lower key))
    (and (org-prefix-at? key "header-args:" 0)
         (string=? (substring key 12 (string-length key)) (org-ascii-lower language)))))
(def (org-alignment-cookie? raw)
  (let* ((value (org-trim raw)) (n (string-length value)))
    (and (>= n 2) (char=? (string-ref value 0) #\<) (char=? (string-ref value (- n 1)) #\>))))
