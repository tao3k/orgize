use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        ORG_ELEMENTS_SQL_COLUMNS, OrgElementKindNamespace, OrgElementPropertyProvenance,
        OrgElementSelector, OrgElementSelectorParseError, OrgElementsHostExecutionOptions,
        OrgElementsIndexCategory, OrgElementsIndexKind, OrgElementsIndexQuery,
        OrgElementsIndexRecord, OrgElementsIndexSummaryValue, ParsedAnnotation,
        PythonDirectiveKind, org_elements_index_query_from_json_str,
        org_elements_index_query_to_json_value,
    },
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "datafusion-sql")]
use datafusion::arrow::array::{Int64Array, StringArray};

#[test]
fn semantic_ast_projects_upstream_org_element_defconsts_match_checked_in_baseline() {
    let upstream = upstream_org_element_defconsts();
    assert_eq!(
        upstream.all_elements,
        string_vec(UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS)
    );
    assert_eq!(
        upstream.greater_elements,
        string_vec(UPSTREAM_ORG_ELEMENT_GREATER_ELEMENTS)
    );
    assert_eq!(
        upstream.all_objects,
        string_vec(UPSTREAM_ORG_ELEMENT_ALL_OBJECTS)
    );
    assert_eq!(
        upstream.recursive_objects,
        string_vec(UPSTREAM_ORG_ELEMENT_RECURSIVE_OBJECTS)
    );
    assert_eq!(
        upstream.affiliated_keywords,
        string_vec(UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS)
    );

    insta::assert_snapshot!(
        serde_json::to_string_pretty(&serde_json::json!({
            "source": "bzg/org-mode .data/org-mode/lisp/org-element.el",
            "allElements": upstream.all_elements,
            "greaterElements": upstream.greater_elements,
            "allObjects": upstream.all_objects,
            "recursiveObjects": upstream.recursive_objects,
            "affiliatedKeywords": upstream.affiliated_keywords,
        }))
        .unwrap()
    );
}

pub(super) fn graph_query_snapshot_records(
    records: Vec<&orgize::ast::OrgElementsIndexRecord<orgize::ast::ParsedAnnotation>>,
) -> Vec<Value> {
    records
        .into_iter()
        .map(|record| {
            serde_json::json!({
                "id": record.id.as_usize(),
                "parentId": record.parent_id.map(|id| id.as_usize()),
                "category": record.category.as_str(),
                "kind": record.kind.as_str(),
                "rawValue": snapshot_summary_value(record.properties.get(":raw-value")),
                "path": snapshot_summary_value(record.properties.get(":path")),
                "outlinePath": record.outline_path,
            })
        })
        .collect()
}

pub(super) fn snapshot_summary_value(value: Option<&OrgElementsIndexSummaryValue>) -> Value {
    match value {
        Some(OrgElementsIndexSummaryValue::Null) | None => Value::Null,
        Some(OrgElementsIndexSummaryValue::Bool(value)) => Value::Bool(*value),
        Some(OrgElementsIndexSummaryValue::Integer(value)) => serde_json::json!(value),
        Some(OrgElementsIndexSummaryValue::Text(value)) => serde_json::json!(value),
        Some(OrgElementsIndexSummaryValue::StringList(value)) => serde_json::json!(value),
    }
}

pub(super) fn string_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| value.to_string()).collect()
}

pub(super) fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

pub(super) fn difference(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.difference(right).cloned().collect()
}

pub(super) fn intersection(left: &BTreeSet<String>, right: &BTreeSet<String>) -> Vec<String> {
    left.intersection(right).cloned().collect()
}

pub(super) fn selected_kind_counts(
    records: &[OrgElementsIndexRecord<ParsedAnnotation>],
    selected_kinds: &[&str],
) -> BTreeMap<String, usize> {
    selected_kinds
        .iter()
        .map(|kind| {
            (
                (*kind).to_string(),
                records
                    .iter()
                    .filter(|record| record.kind.as_str() == *kind)
                    .count(),
            )
        })
        .collect()
}

#[derive(Debug)]
pub(super) struct UpstreamOrgElementDefconsts {
    pub(super) all_elements: Vec<String>,
    pub(super) greater_elements: Vec<String>,
    pub(super) all_objects: Vec<String>,
    pub(super) recursive_objects: Vec<String>,
    pub(super) affiliated_keywords: Vec<String>,
}

pub(super) fn upstream_org_element_defconsts() -> UpstreamOrgElementDefconsts {
    let source = include_str!("../../../.data/org-mode/lisp/org-element.el");
    UpstreamOrgElementDefconsts {
        all_elements: elisp_defconst_quoted_list(source, "org-element-all-elements"),
        greater_elements: elisp_defconst_quoted_list(source, "org-element-greater-elements"),
        all_objects: elisp_defconst_quoted_list(source, "org-element-all-objects"),
        recursive_objects: elisp_defconst_quoted_list(source, "org-element-recursive-objects"),
        affiliated_keywords: elisp_defconst_quoted_list(source, "org-element-affiliated-keywords"),
    }
}

pub(super) fn elisp_defconst_quoted_list(source: &str, name: &str) -> Vec<String> {
    let marker = format!("(defconst {name}");
    let defconst = source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing upstream defconst `{name}`"))
        .1;
    let body_start = defconst
        .find("'(")
        .unwrap_or_else(|| panic!("missing quoted list for upstream defconst `{name}`"))
        + 2;
    let body = quoted_list_body(&defconst[body_start..]);
    elisp_list_values(body)
}

pub(super) fn quoted_list_body(source: &str) -> &str {
    let mut depth = 1usize;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &source[..index];
                }
            }
            _ => {}
        }
    }
    source
}

pub(super) fn elisp_list_values(source: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '"' {
            let mut value = String::new();
            while let Some(ch) = chars.next() {
                match ch {
                    '"' => break,
                    '\\' => {
                        if let Some(escaped) = chars.next() {
                            value.push(escaped);
                        }
                    }
                    _ => value.push(ch),
                }
            }
            values.push(value);
            continue;
        }
        let mut value = ch.to_string();
        while let Some(next) = chars.peek().copied() {
            if next.is_whitespace() || next == '(' || next == ')' {
                break;
            }
            value.push(next);
            chars.next();
        }
        values.push(value);
    }
    values
}

pub(super) const UPSTREAM_ORG_ELEMENT_ALL_ELEMENTS: &[&str] = &[
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

pub(super) const UPSTREAM_ORG_ELEMENT_ALL_OBJECTS: &[&str] = &[
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

pub(super) const UPSTREAM_ORG_ELEMENT_GREATER_ELEMENTS: &[&str] = &[
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

pub(super) const UPSTREAM_ORG_ELEMENT_RECURSIVE_OBJECTS: &[&str] = &[
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

pub(super) const UPSTREAM_ORG_ELEMENT_AFFILIATED_KEYWORDS: &[&str] = &[
    "CAPTION", "DATA", "HEADER", "HEADERS", "LABEL", "NAME", "PLOT", "RESNAME", "RESULT",
    "RESULTS", "SOURCE", "SRCNAME", "TBLNAME",
];

pub(super) const UPSTREAM_ORG_ELEMENT_STANDARD_PROPERTIES: &[&str] = &[
    ":begin",
    ":post-affiliated",
    ":contents-begin",
    ":contents-end",
    ":end",
    ":post-blank",
    ":secondary",
    ":mode",
    ":granularity",
    ":cached",
    ":org-element--cache-sync-key",
    ":robust-begin",
    ":robust-end",
    ":true-level",
    ":buffer",
    ":deferred",
    ":structure",
    ":parent",
];

pub(super) const ORG_ELEMENT_INTENTIONALLY_UNMAPPED_STANDARD_PROPERTIES: &[&str] = &[
    ":buffer",
    ":cached",
    ":deferred",
    ":granularity",
    ":mode",
    ":org-element--cache-sync-key",
    ":robust-begin",
    ":robust-end",
    ":secondary",
    ":structure",
];
