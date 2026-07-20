//! Core facade for Org elements query expression parsing and compilation.

use super::core_contract::{
    apply_relative_scope, compile_contract_sequence, compile_query_expression,
};
use super::core_parser::{lower_root, parse_query_expression_syntax, unquote_query_string};
pub use super::core_types::OrgElementsQueryExpressionError;
pub(super) use super::core_types::{FieldKind, QueryExpr, RelativeKind, list_head};
use crate::ast::{
    OrgContractBinding, OrgContractExpectation, OrgContractQuery, OrgElementsIndexCategory,
    OrgElementsIndexKind, OrgElementsIndexQuery, OrgElementsIndexSummaryValue,
};

pub fn org_elements_index_query_from_expr_str(
    value: &str,
) -> Result<OrgElementsIndexQuery, OrgElementsQueryExpressionError> {
    let expressions = parse_expressions(value).ok_or_else(|| {
        OrgElementsQueryExpressionError::new("invalid Org elements query expression syntax")
    })?;
    super::index::compile_index_query_expressions(&expressions).ok_or_else(|| {
        OrgElementsQueryExpressionError::new("unsupported Org elements query expression")
    })
}

/// Parses one expression block as a query-only Org elements IR.
pub(in crate::ast) fn parse_org_elements_query_expression_block(
    value: &str,
) -> Option<OrgContractQuery> {
    let parsed = parse_expressions(value).and_then(|expressions| match expressions.as_slice() {
        [expression] => compile_query_expression(expression),
        [] => None,
        expressions => {
            let mut query = OrgContractQuery::default();
            for expression in expressions {
                merge_query(&mut query, compile_query_expression(expression)?);
            }
            Some(query)
        }
    });
    parsed.or_else(|| parse_legacy_org_elements_query_block(value))
}

/// Parses one expression block as a contract assertion.
pub(in crate::ast) fn parse_org_contract_expression_block(
    value: &str,
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    let expressions = parse_expressions(value)?;
    match expressions.as_slice() {
        [expression] => compile_contract_expression(expression),
        _ => compile_contract_sequence(&expressions),
    }
}

pub(in crate::ast) fn apply_org_elements_query_kind(kind: &str, query: &mut OrgContractQuery) {
    let kind = kind.trim().trim_matches('"');
    match kind {
        "org-data" => {
            query.category = Some(OrgElementsIndexCategory::Document);
            query.kind = Some(OrgElementsIndexKind::new("org-data"));
        }
        "headline" => {
            query.category = Some(OrgElementsIndexCategory::Section);
            query.kind = Some(OrgElementsIndexKind::new("headline"));
        }
        "node-property" => {
            query.category = Some(OrgElementsIndexCategory::Property);
            query.kind = Some(OrgElementsIndexKind::new("node-property"));
        }
        "keyword" => {
            query.category = Some(OrgElementsIndexCategory::Keyword);
            query.kind = Some(OrgElementsIndexKind::new("keyword"));
        }
        "link" | "timestamp" | "bold" | "italic" | "underline" | "strike-through"
        | "superscript" | "subscript" | "code" | "verbatim" | "target" | "radio-target"
        | "footnote-reference" | "citation" | "inline-src-block" | "inline-babel-call"
        | "macro" | "plain-text" | "table-cell" => {
            query.category = Some(OrgElementsIndexCategory::Object);
            query.kind = Some(OrgElementsIndexKind::new(kind));
        }
        _ => {
            query.category = Some(OrgElementsIndexCategory::Element);
            query.kind = Some(OrgElementsIndexKind::new(kind));
        }
    }
}

pub(in crate::ast) fn org_elements_query_summary_value(
    value: &str,
) -> OrgElementsIndexSummaryValue {
    match value {
        "t" | "true" => OrgElementsIndexSummaryValue::Bool(true),
        "nil" | "false" => OrgElementsIndexSummaryValue::Bool(false),
        "null" => OrgElementsIndexSummaryValue::Null,
        _ => value
            .parse::<i64>()
            .map(OrgElementsIndexSummaryValue::Integer)
            .unwrap_or_else(|_| OrgElementsIndexSummaryValue::Text(value.to_string())),
    }
}

fn parse_legacy_org_elements_query_block(value: &str) -> Option<OrgContractQuery> {
    let mut query = OrgContractQuery::default();
    let mut parsed = false;
    for line in value.lines().map(strip_legacy_query_comment) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=')?;
        let key = key.trim();
        let value = unquote_legacy_query_value(value.trim())?;
        apply_legacy_query_assignment(&mut query, key, &value)?;
        parsed = true;
    }
    parsed.then_some(query)
}

fn apply_legacy_query_assignment(
    query: &mut OrgContractQuery,
    key: &str,
    value: &str,
) -> Option<()> {
    match key {
        "category" => {
            query.category = Some(OrgElementsIndexCategory::from_label(value)?);
        }
        "kind" => {
            apply_org_elements_query_kind(value, query);
        }
        "within" | "descendant_of" | "descendant-of" => {
            apply_relative_scope(query, RelativeKind::Descendant, value);
        }
        "child_of" | "child-of" => {
            apply_relative_scope(query, RelativeKind::Child, value);
        }
        "at" => {
            apply_relative_scope(query, RelativeKind::At, value);
        }
        "name" | "affiliated_name" | "affiliated-name" => {
            query.affiliated_name = Some(value.to_string());
        }
        "context" => {
            query.context = Some(value.to_string());
        }
        "limit" => {
            query.limit = Some(value.parse::<usize>().ok()?);
        }
        _ => {
            if let Some(field) = key.strip_prefix("summary.") {
                query
                    .summary_equals
                    .push((field.to_string(), value.to_string()));
            } else if let Some(field) = key
                .strip_prefix("summary_contains.")
                .or_else(|| key.strip_prefix("summary-contains."))
            {
                query
                    .summary_contains
                    .push((field.to_string(), value.to_string()));
            } else if let Some(field) = key.strip_prefix("property.") {
                query
                    .property_equals
                    .push((field.to_string(), value.to_string()));
            } else {
                let field = key
                    .strip_prefix("property_contains.")
                    .or_else(|| key.strip_prefix("property-contains."))?;
                query
                    .property_contains
                    .push((field.to_string(), value.to_string()));
            }
        }
    }
    Some(())
}

fn strip_legacy_query_comment(line: &str) -> &str {
    let trimmed = line.trim();
    if trimmed.starts_with('#') {
        return "";
    }
    line.split_once(" #").map_or(line, |(before, _)| before)
}

fn unquote_legacy_query_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with('"') {
        return unquote_query_string(value);
    }
    Some(value.to_string())
}

fn parse_expressions(value: &str) -> Option<Vec<QueryExpr>> {
    let syntax = parse_query_expression_syntax(value)?;
    lower_root(&syntax)
}
use super::core_contract::{compile_contract_expression, merge_query};
