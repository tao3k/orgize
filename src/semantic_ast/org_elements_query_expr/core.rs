//! Core facade for Org elements query expression parsing and compilation.

use super::core_contract::{compile_contract_sequence, compile_query_expression};
use super::core_parser::{lower_root, parse_query_expression_syntax};
pub use super::core_types::OrgElementsQueryExpressionError;
pub(super) use super::core_types::{FieldKind, QueryExpr, list_head};
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
    parsed.or_else(|| {
        let normalized = normalize_legacy_query_expression(value)?;
        parse_expressions(&normalized).and_then(|expressions| match expressions.as_slice() {
            [expression] => compile_query_expression(expression),
            [] => None,
            expressions => {
                let mut query = OrgContractQuery::default();
                for expression in expressions {
                    merge_query(&mut query, compile_query_expression(expression)?);
                }
                Some(query)
            }
        })
    })
}

fn normalize_legacy_query_expression(value: &str) -> Option<String> {
    let mut kind = None;
    let mut within = None;
    let mut summary = None;
    let mut saw_condition = false;

    for raw_line in value.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, raw_value) = line.split_once('=')?;
        let key = key.trim();
        let raw_value = raw_value.trim();
        saw_condition = true;
        match key {
            "category" => {}
            "kind" => kind = Some(raw_value.trim_matches('"').to_string()),
            "within" => within = Some(raw_value.trim_matches('"').to_string()),
            _ if key.starts_with("summary.") => {
                summary = Some((
                    key.trim_start_matches("summary.").to_string(),
                    raw_value.to_string(),
                ));
            }
            _ => return None,
        }
    }

    if !saw_condition {
        return None;
    }
    let kind = kind?;
    let mut expression = format!("({kind}");
    if let Some(within) = within {
        expression.push_str(" :descendant-of ");
        expression.push_str(&within);
    }
    if let Some((field, value)) = summary {
        expression.push_str(" :summary (");
        expression.push_str(&field);
        expression.push(' ');
        expression.push_str(&value);
        expression.push(')');
    }
    expression.push(')');
    Some(expression)
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

fn parse_expressions(value: &str) -> Option<Vec<QueryExpr>> {
    let syntax = parse_query_expression_syntax(value)?;
    lower_root(&syntax)
}
use super::core_contract::{compile_contract_expression, merge_query};
