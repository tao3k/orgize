//! Build-time helpers for generated `orgize` source artifacts.

mod builtin_lint_contracts;
mod source_revision;

pub use builtin_lint_contracts::write_builtin_lint_contract_manifest;
pub use source_revision::write_source_revision;
