//! Command-line interface boundary for the `orgize` binary.

mod document;
mod document_index;
mod document_json;
mod driver;
mod eval;

pub use driver::run_from_env;
