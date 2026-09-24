//! Build-time helpers for generated `orgize` source artifacts.

mod builtin_lint_contracts;
mod org_aot_functions;
mod source_revision;

pub use builtin_lint_contracts::write_builtin_lint_contract_manifest;
pub use org_aot_functions::write_org_aot_events;
pub use org_aot_functions::write_org_aot_functions;
pub use source_revision::write_source_revision;
