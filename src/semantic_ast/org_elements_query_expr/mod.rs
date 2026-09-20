//! Elisp-style Org elements query expressions.
//!
//! The expression surface is calibrated against
//! `.data/org-mode/lisp/org-element-ast.el`: Org syntax nodes are selected by
//! node type plus plist-like properties, and traversal follows contents and
//! lineage. Secondary property contents are queryable when the parser projects
//! them into summary or property facts.

mod core;
mod core_contract;
mod core_parser;
mod core_predicate;
mod core_types;
mod index;
mod surface;

use core::{FieldKind, QueryExpr, list_head};
pub use core::{OrgElementsQueryExpressionError, org_elements_index_query_from_expr_str};
pub(in crate::ast) use core::{
    apply_org_elements_query_kind, parse_org_contract_expression_block,
    parse_org_elements_query_expression_block,
};
pub(crate) use core::{
    parse_org_contract_document_reference_resolution_block,
    parse_org_contract_node_reference_resolution_block,
    parse_org_contract_pair_document_equality_block, parse_org_contract_pair_node_equality_block,
};
use core_predicate::{compile_predicate_expression, expression_summary_value, parse_field_ref};
pub use surface::{
    ORG_ELEMENTS_QUERY_EXPRESSION_EXAMPLES, ORG_ELEMENTS_QUERY_EXPRESSION_SURFACE_GUIDE,
};
