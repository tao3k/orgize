//! Scheme owns expression syntax and cooked values; Rust projects their ABI.

use super::core_types::QueryExpr;

#[path = "../../../languages/org/v1/modules/org-contract/generated/parser.rs"]
#[rustfmt::skip]
mod grammar;

pub(super) fn parse_query_expression_values(value: &str) -> Option<Vec<QueryExpr>> {
    crate::org_aot::parse_native_expression_values(value, &grammar::LANGUAGE).ok()
}

pub(super) fn parse_contract_expression_values(value: &str) -> Option<Vec<QueryExpr>> {
    crate::org_aot::parse_native_contract_values(value, &grammar::LANGUAGE).ok()
}

#[cfg(test)]
pub(super) fn parse_query_expression_syntax(value: &str) -> Option<gerbil_parser_rowan::Parse> {
    crate::org_aot::parse_native_expression(value, &grammar::LANGUAGE).ok()
}

#[cfg(test)]
#[path = "../../../tests/unit/org_contract_native_cst.rs"]
pub(crate) mod tests;
