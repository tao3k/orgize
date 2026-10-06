//! Native event transport interface; parsing semantics remain Scheme-owned.
mod batch;
mod transport;

pub(crate) use batch::{
    MAX_DOCUMENTS as BATCH_MAX_DOCUMENTS, MAX_SOURCE_BYTES as BATCH_MAX_SOURCE_BYTES,
};
#[cfg(test)]
pub(super) use transport::parse_expression_events;
pub(super) use transport::{
    evaluate_contract, expand_macro_fields, initialize_owner, parse_contract_values, parse_events,
    parse_events_batch, parse_expectation_values, parse_expression_values, semantic_fields,
};
