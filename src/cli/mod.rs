//! Command-line interface boundary for the `orgize` binary.

mod driver;
mod eval;
mod org_contract_registry;
pub(crate) mod org_contract_trace;

pub use driver::run_from_env;
