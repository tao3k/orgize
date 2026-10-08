//! Scheme owns expression syntax and cooked values; Rust projects their ABI.

use super::core_types::QueryExpr;

#[path = "../../../languages/org/modules/org-contract/generated/parser.rs"]
#[rustfmt::skip]
mod grammar;

pub(super) fn parse_query_expression_values(value: &str) -> Option<Vec<QueryExpr>> {
    crate::org_aot::parse_native_expression_values(value, &grammar::LANGUAGE).ok()
}

#[cfg(test)]
pub(super) fn parse_query_expression_syntax(value: &str) -> Option<gerbil_parser_runtime::Parse> {
    crate::org_aot::parse_native_expression(value, &grammar::LANGUAGE).ok()
}

#[cfg(test)]
#[path = "../../../tests/unit/org_contract_native_cst.rs"]
pub(crate) mod tests;
