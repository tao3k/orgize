;;; -*- Gerbil -*-
;;; Org v1 element and object inventory owned by the Scheme language pack.
;;; The upstream Org mode revision is comparison evidence, not executable input.

(export +org-element-kinds+ +org-greater-element-kinds+
        +org-object-kinds+ +org-recursive-object-kinds+
        +org-affiliated-keywords+ +org-object-restrictions+
        +org-secondary-values+ org-object-allowed?
        org-secondary-value?)

(def +org-element-kinds+
  '("babel-call" "center-block" "clock" "comment" "comment-block"
    "diary-sexp" "drawer" "dynamic-block" "example-block" "export-block"
    "fixed-width" "footnote-definition" "headline" "horizontal-rule"
    "inlinetask" "item" "keyword" "latex-environment" "node-property"
    "paragraph" "plain-list" "planning" "property-drawer" "quote-block"
    "section" "special-block" "src-block" "table" "table-row"
    "verse-block"))

(def +org-greater-element-kinds+
  '("center-block" "drawer" "dynamic-block" "footnote-definition"
    "headline" "inlinetask" "item" "plain-list" "property-drawer"
    "quote-block" "section" "special-block" "table" "org-data"))

(def +org-object-kinds+
  '("bold" "citation" "citation-reference" "code" "entity"
    "export-snippet" "footnote-reference" "inline-babel-call"
    "inline-src-block" "italic" "line-break" "latex-fragment" "link"
    "macro" "radio-target" "statistics-cookie" "strike-through"
    "subscript" "superscript" "table-cell" "target" "timestamp"
    "underline" "verbatim"))

(def +org-recursive-object-kinds+
  '("bold" "citation" "footnote-reference" "italic" "link"
    "subscript" "radio-target" "strike-through" "superscript"
    "table-cell" "underline"))

(def +org-affiliated-keywords+
  '("CAPTION" "DATA" "HEADER" "HEADERS" "LABEL" "NAME" "PLOT"
    "RESNAME" "RESULT" "RESULTS" "SOURCE" "SRCNAME" "TBLNAME"))

;; Object parsing depends on the containing element.  These are language-pack
;; data, not a global search for every delimiter in every text span.
(def +org-minimal-objects+
  '("bold" "code" "entity" "italic" "latex-fragment" "strike-through"
    "subscript" "superscript" "underline" "verbatim"))

(def +org-standard-objects+
  (filter (lambda (kind)
            (not (member kind '("citation-reference" "table-cell"))))
          +org-object-kinds+))

(def +org-no-line-break-objects+
  (filter (lambda (kind) (not (equal? kind "line-break")))
          +org-standard-objects+))

(def +org-object-restrictions+
  `(("bold" . ,+org-standard-objects+)
    ("citation" . ("citation-reference"))
    ("citation-reference" .
     ,(filter (lambda (kind)
                (not (member kind '("citation" "footnote-reference" "link"))))
              +org-no-line-break-objects+))
    ("footnote-reference" . ,+org-standard-objects+)
    ("headline" . ,+org-no-line-break-objects+)
    ("inlinetask" . ,+org-no-line-break-objects+)
    ("italic" . ,+org-standard-objects+)
    ("item" . ,+org-no-line-break-objects+)
    ("keyword" .
     ,(filter (lambda (kind) (not (equal? kind "footnote-reference")))
              +org-standard-objects+))
    ("link" . ,(append '("export-snippet" "inline-babel-call"
                          "inline-src-block" "macro" "statistics-cookie")
                        +org-minimal-objects+))
    ("paragraph" . ,+org-standard-objects+)
    ("radio-target" . ,+org-minimal-objects+)
    ("strike-through" . ,+org-standard-objects+)
    ("subscript" . ,+org-standard-objects+)
    ("superscript" . ,+org-standard-objects+)
    ("table-cell" .
     ,(append '("citation" "export-snippet" "footnote-reference" "link"
                "macro" "radio-target" "target" "timestamp")
              +org-minimal-objects+))
    ("table-row" . ("table-cell"))
    ("underline" . ,+org-standard-objects+)
    ("verse-block" . ,+org-standard-objects+)))

(def +org-secondary-values+
  '(("citation" . ("prefix" "suffix"))
    ("headline" . ("title"))
    ("inlinetask" . ("title"))
    ("item" . ("tag"))
    ("citation-reference" . ("prefix" "suffix"))))

(def (org-object-allowed? container object)
  (let (entry (assoc container +org-object-restrictions+))
    (and entry (if (member object (cdr entry)) #t #f))))

(def (org-secondary-value? container field)
  (let (entry (assoc container +org-secondary-values+))
    (and entry (if (member field (cdr entry)) #t #f))))
