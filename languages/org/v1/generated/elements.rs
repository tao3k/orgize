// Generated from languages/org/v1/elements.ss. Do not edit.

pub const ELEMENTS_DIGEST: &str =
    "sha256:96846ea16ab828c06cefe9b52f7e81c093474cecd06910bba032ce742bdea2e4";

#[rustfmt::skip]
pub const ORG_ELEMENT_KINDS: &[&str] = &[
    "babel-call",
    "center-block",
    "clock",
    "comment",
    "comment-block",
    "diary-sexp",
    "drawer",
    "dynamic-block",
    "example-block",
    "export-block",
    "fixed-width",
    "footnote-definition",
    "headline",
    "horizontal-rule",
    "inlinetask",
    "item",
    "keyword",
    "latex-environment",
    "node-property",
    "paragraph",
    "plain-list",
    "planning",
    "property-drawer",
    "quote-block",
    "section",
    "special-block",
    "src-block",
    "table",
    "table-row",
    "verse-block",
];

#[rustfmt::skip]
pub const ORG_GREATER_ELEMENT_KINDS: &[&str] = &[
    "center-block",
    "drawer",
    "dynamic-block",
    "footnote-definition",
    "headline",
    "inlinetask",
    "item",
    "plain-list",
    "property-drawer",
    "quote-block",
    "section",
    "special-block",
    "table",
    "org-data",
];

#[rustfmt::skip]
pub const ORG_OBJECT_KINDS: &[&str] = &[
    "bold",
    "citation",
    "citation-reference",
    "code",
    "entity",
    "export-snippet",
    "footnote-reference",
    "inline-babel-call",
    "inline-src-block",
    "italic",
    "line-break",
    "latex-fragment",
    "link",
    "macro",
    "radio-target",
    "statistics-cookie",
    "strike-through",
    "subscript",
    "superscript",
    "table-cell",
    "target",
    "timestamp",
    "underline",
    "verbatim",
];

#[rustfmt::skip]
pub const ORG_RECURSIVE_OBJECT_KINDS: &[&str] = &[
    "bold",
    "citation",
    "footnote-reference",
    "italic",
    "link",
    "subscript",
    "radio-target",
    "strike-through",
    "superscript",
    "table-cell",
    "underline",
];

#[rustfmt::skip]
pub const ORG_AFFILIATED_KEYWORDS: &[&str] = &[
    "CAPTION",
    "DATA",
    "HEADER",
    "HEADERS",
    "LABEL",
    "NAME",
    "PLOT",
    "RESNAME",
    "RESULT",
    "RESULTS",
    "SOURCE",
    "SRCNAME",
    "TBLNAME",
];

#[rustfmt::skip]
pub const ORG_OBJECT_RESTRICTIONS: &[(&str, &[&str])] = &[
    ("bold", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("citation", &["citation-reference", ]),
    ("citation-reference", &["bold", "code", "entity", "export-snippet", "inline-babel-call", "inline-src-block", "italic", "latex-fragment", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("footnote-reference", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("headline", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("inlinetask", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("italic", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("item", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("keyword", &["bold", "citation", "code", "entity", "export-snippet", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("link", &["export-snippet", "inline-babel-call", "inline-src-block", "macro", "statistics-cookie", "bold", "code", "entity", "italic", "latex-fragment", "strike-through", "subscript", "superscript", "underline", "verbatim", ]),
    ("paragraph", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("radio-target", &["bold", "code", "entity", "italic", "latex-fragment", "strike-through", "subscript", "superscript", "underline", "verbatim", ]),
    ("strike-through", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("subscript", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("superscript", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("table-cell", &["citation", "export-snippet", "footnote-reference", "link", "macro", "radio-target", "target", "timestamp", "bold", "code", "entity", "italic", "latex-fragment", "strike-through", "subscript", "superscript", "underline", "verbatim", ]),
    ("table-row", &["table-cell", ]),
    ("underline", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
    ("verse-block", &["bold", "citation", "code", "entity", "export-snippet", "footnote-reference", "inline-babel-call", "inline-src-block", "italic", "line-break", "latex-fragment", "link", "macro", "radio-target", "statistics-cookie", "strike-through", "subscript", "superscript", "target", "timestamp", "underline", "verbatim", ]),
];

#[rustfmt::skip]
pub const ORG_SECONDARY_VALUES: &[(&str, &[&str])] = &[
    ("citation", &["prefix", "suffix", ]),
    ("headline", &["title", ]),
    ("inlinetask", &["title", ]),
    ("item", &["tag", ]),
    ("citation-reference", &["prefix", "suffix", ]),
];
