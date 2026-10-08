//! Contract source owner interface.
mod core;
mod native_blocks;
mod source_plan;
pub(crate) use core::{contract_block_syntax_error, contract_source_blocks};
pub use core::{
    parse_contract_reference, parse_contract_reference_from_source, parse_contract_references,
    parse_contracts_from_document, validate_contract_source,
};
