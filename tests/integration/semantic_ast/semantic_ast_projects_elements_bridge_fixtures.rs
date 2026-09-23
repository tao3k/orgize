use orgize::ast::{OrgElementsIndexRecord, OrgElementsIndexSummaryValue, ParsedAnnotation};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[path = "../../../languages/org/v1/generated/elements.rs"]
mod generated_elements;

pub(super) use generated_elements::{
    ORG_AFFILIATED_KEYWORDS, ORG_ELEMENT_KINDS, ORG_GREATER_ELEMENT_KINDS, ORG_OBJECT_KINDS,
    ORG_RECURSIVE_OBJECT_KINDS,
};

#[test]
fn semantic_ast_projects_scheme_element_catalog_matches_approved_baseline() {
    insta::assert_snapshot!(
        "scheme_element_catalog",
        serde_json::to_string_pretty(&serde_json::json!({
            "source": "languages/org/v1/elements.ss",
            "allElements": ORG_ELEMENT_KINDS,
            "greaterElements": ORG_GREATER_ELEMENT_KINDS,
            "allObjects": ORG_OBJECT_KINDS,
            "recursiveObjects": ORG_RECURSIVE_OBJECT_KINDS,
            "affiliatedKeywords": ORG_AFFILIATED_KEYWORDS,
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
