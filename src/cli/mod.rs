//! Command-line interface boundary for the `orgize` binary.

mod driver;
mod driver_fmt_lint;
mod driver_paths;
mod driver_sdd;
mod driver_tasks;
mod driver_usage;
mod eval;
mod org_contract_registry;
pub(crate) mod org_contract_trace;

pub(crate) use driver::run_args;
pub use driver::run_from_env;
