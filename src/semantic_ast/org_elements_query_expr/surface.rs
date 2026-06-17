//! Agent-facing query surface guide for Org elements expressions.

/// Searchable guide entries for the Org elements query expression surface.
///
/// These strings are intentionally plain and redundant: ASP source indexing can
/// use them as discoverability anchors for relation aliases, contents traversal,
/// lineage traversal, secondary property handling, predicate forms, and JSON
/// index lowering behavior.
pub const ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE: &[&str] = &[
    "root: (org-elements-query ...) and (query ...) group forms with AND semantics",
    "node type: (kind TYPE), (type TYPE), and kind sugar such as (headline ...) select Org element kinds",
    "property: (property KEY VALUE), (property-contains KEY TEXT), and :property plist entries query Org element properties",
    "summary: (summary KEY VALUE), (summary-contains KEY TEXT), and :summary plist entries query projected index summary facts",
    "relation: (child-of ID), (descendant-of ID), (ancestor-of ID), and (at ID) query indexed Org element relations",
    "contents: (contents-of ID) maps to direct child contents and (within-contents-of ID) maps to descendant contents",
    "lineage: (lineage-of ID) maps to ancestor lineage, matching Org element lineage traversal",
    "secondary: secondary property contents are queryable only after parser projection into summary or property facts",
    "predicate: (predicate (and ...)), (predicate (or ...)), (predicate (not ...)), (= (summary KEY) VALUE), and (contains (property KEY) TEXT)",
    "json index: expression lowering produces OrgElementsIndexQuery, the same query model used by JSON index packets",
];

/// Runnable examples for the Org elements query expression guide.
///
/// Unit tests parse every example so agent-facing guide snippets remain aligned
/// with the actual index lowering implementation.
pub const ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES: &[&str] = &[
    r#"(org-elements-query (kind headline) (property :CUSTOM_ID "plan"))"#,
    r#"(org-elements-query (summary-contains text "evidence") (contents-of 1))"#,
    r#"(org-elements-query (within-contents-of 2) (lineage-of 3))"#,
    r#"(org-elements-query (predicate (or (kind link) (= (summary hasText) t))))"#,
    r#"(headline :property (:CUSTOM_ID "plan") :within-contents-of 4 :limit 5)"#,
];
