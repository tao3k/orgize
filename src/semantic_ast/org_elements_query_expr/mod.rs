//! Org elements query expressions over parser-owned syntax.
//!
//! Nodes are selected by type plus plist-like properties, and traversal
//! follows contents and lineage. Secondary property contents are queryable
//! when the parser projects them into summary or property facts. The Org
//! element inventory is declared in `languages/org/modules/org-elements/catalog.ss`.

mod core;
mod core_contract;
mod core_parser;
mod core_predicate;
mod core_types;
mod index;
mod surface;

#[cfg(test)]
pub(crate) use core_parser::tests::NATIVE_CASES as EXPRESSION_NATIVE_CASES;

use core::{FieldKind, QueryExpr, list_head};
pub use core::{OrgElementsQueryExpressionError, org_elements_index_query_from_expr_str};
pub(in crate::ast) use core::{
    apply_org_elements_query_kind, compile_contract_values, compile_query_values,
    contract_values_are_admitted, query_values_are_admitted, selector_plist_properties,
    selector_properties_from_values,
};
pub(crate) use core::{
    parse_org_contract_pair_document_equality_block, parse_org_contract_pair_node_equality_block,
    parse_org_contract_workspace_reference_block,
};
use core_predicate::{compile_predicate_expression, expression_summary_value, parse_field_ref};
pub use surface::{
    ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES, ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE,
};
