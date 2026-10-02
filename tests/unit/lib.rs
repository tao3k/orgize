//! Test helpers mounted into the crate during `cargo test --lib`.

#[path = "aot_affiliation.rs"]
mod aot_affiliation;
#[path = "aot_projection.rs"]
mod aot_projection;
#[path = "document_command.rs"]
mod document_command;
#[path = "document_source_selection.rs"]
mod document_source_selection;
#[path = "elements_bridge_query_json.rs"]
mod elements_bridge_query_json;
#[path = "lint_metadata.rs"]
mod lint_metadata;
#[path = "org_contract_evaluation.rs"]
mod org_contract_evaluation;
#[path = "org_contract_reference.rs"]
mod org_contract_reference;
#[path = "org_contract_source_validation.rs"]
mod org_contract_source_validation;
#[path = "org_elements_query_expr.rs"]
mod org_elements_query_expr;
