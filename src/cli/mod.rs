//! Command-line interface boundary for the `orgize` binary.

mod document;
mod driver;
mod eval;

pub use driver::run_from_env;
