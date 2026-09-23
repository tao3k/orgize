;;; -*- Gerbil -*-
;;; Org v1 element and object inventory owned by the Scheme language pack.
;;; The upstream Org mode revision is comparison evidence, not executable input.

(export +org-element-kinds+ +org-greater-element-kinds+
        +org-object-kinds+ +org-recursive-object-kinds+
        +org-affiliated-keywords+)

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
