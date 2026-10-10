//! Core facade for Org elements query expression parsing and compilation.

use super::core_contract::{
    compile_contract_sequence, compile_pair_document_equality, compile_pair_node_equality,
    compile_query_expression, compile_workspace_reference,
};
use super::core_parser::parse_query_expression_values;
pub use super::core_types::OrgElementsQueryExpressionError;
pub(super) use super::core_types::{FieldKind, QueryExpr, list_head};
use crate::ast::{
    OrgContractBinding, OrgContractExpectation, OrgContractPairDocumentEquality,
    OrgContractPairNodeEquality, OrgContractQuery, OrgContractWorkspaceReference,
    OrgElementsIndexCategory, OrgElementsIndexKind, OrgElementsIndexQuery,
    OrgElementsIndexSummaryValue,
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

pub(in crate::ast) fn selector_plist_properties(
    value: &str,
) -> Result<Vec<(String, String)>, crate::ast::OrgElementSelectorParseError> {
    use crate::ast::OrgElementSelectorParseError::InvalidShape;
    let expressions = parse_expressions(value).ok_or(InvalidShape)?;
    selector_properties_from_values(&expressions)
}

pub(in crate::ast) fn selector_properties_from_values(
    expressions: &[QueryExpr],
) -> Result<Vec<(String, String)>, crate::ast::OrgElementSelectorParseError> {
    use crate::ast::OrgElementSelectorParseError::{InvalidShape, OddPropertyList};
    let [QueryExpr::List(items)] = expressions else {
        return Err(InvalidShape);
    };
    let [QueryExpr::Atom(head), QueryExpr::List(properties)] = items.as_slice() else {
        return Err(InvalidShape);
    };
    if head != ":org-element" {
        return Err(InvalidShape);
    }
    if properties.len() % 2 != 0 {
        return Err(OddPropertyList);
    }
    properties
        .chunks(2)
        .map(|pair| {
            let key = pair[0].as_atom().ok_or(InvalidShape)?.to_string();
            let value = pair[1].as_text().ok_or(InvalidShape)?;
            Ok((key, value))
        })
        .collect()
}

pub(in crate::ast) fn query_values_are_admitted(expressions: &[QueryExpr]) -> bool {
    super::index::compile_index_query_expressions(expressions).is_some()
        || (!expressions.is_empty()
            && expressions
                .iter()
                .all(|expression| compile_query_expression(expression).is_some()))
}

pub(in crate::ast) fn compile_query_values(expressions: &[QueryExpr]) -> Option<OrgContractQuery> {
    match expressions {
        [expression] => compile_query_expression(expression),
        [] => None,
        expressions => {
            let mut query = OrgContractQuery::default();
            for expression in expressions {
                merge_query(&mut query, compile_query_expression(expression)?);
            }
            Some(query)
        }
    }
}

pub(in crate::ast) fn contract_values_are_admitted(
    normalized: Option<&[QueryExpr]>,
    raw: Option<&[QueryExpr]>,
) -> bool {
    if normalized.and_then(compile_contract_sequence).is_some() {
        return true;
    }
    match raw {
        Some([expression]) => {
            compile_pair_node_equality(expression).is_some()
                || compile_pair_document_equality(expression).is_some()
                || compile_workspace_reference(expression).is_some()
        }
        _ => false,
    }
}

pub(in crate::ast) fn compile_contract_values(
    expressions: &[QueryExpr],
) -> Option<(
    Vec<OrgContractBinding>,
    OrgContractQuery,
    OrgContractExpectation,
)> {
    compile_contract_sequence(expressions)
}

pub(crate) fn parse_org_contract_pair_node_equality_block(
    value: &str,
) -> Option<OrgContractPairNodeEquality> {
    let expressions = parse_expressions(value)?;
    match expressions.as_slice() {
        [expression] => compile_pair_node_equality(expression),
        _ => None,
    }
}

pub(crate) fn parse_org_contract_pair_document_equality_block(
    value: &str,
) -> Option<OrgContractPairDocumentEquality> {
    let expressions = parse_expressions(value)?;
    match expressions.as_slice() {
        [expression] => compile_pair_document_equality(expression),
        _ => None,
    }
}

pub(crate) fn parse_org_contract_workspace_reference_block(
    value: &str,
) -> Option<OrgContractWorkspaceReference> {
    let expressions = parse_expressions(value)?;
    match expressions.as_slice() {
        [expression] => compile_workspace_reference(expression),
        _ => None,
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
    parse_query_expression_values(value)
}
use super::core_contract::merge_query;
