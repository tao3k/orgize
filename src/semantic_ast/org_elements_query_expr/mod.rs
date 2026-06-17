//! Elisp-style Org elements query expressions.
//!
//! The expression surface is calibrated against
//! `.data/org-mode/lisp/org-element-ast.el`: Org syntax nodes are selected by
//! node type plus plist-like properties, and traversal follows contents,
//! secondary properties, and lineage.

mod core;
mod index;

use core::{
    FieldKind, QueryExpr, compile_predicate_expression, expression_summary_value, list_head,
    parse_field_ref,
};
pub use core::{OrgElementsQueryExpressionError, org_elements_index_query_from_expr_str};
pub(in crate::ast) use core::{
    apply_org_elements_query_kind, org_elements_query_summary_value,
    parse_org_contract_expression_block, parse_org_elements_query_expression_block,
};
