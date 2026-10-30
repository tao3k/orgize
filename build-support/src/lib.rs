//! Build-time helpers for generated `orgize` source artifacts.

mod ffi_bundle;
mod org_program;
mod source_revision;

pub use org_program::write_org_program;
pub use source_revision::write_source_revision;
