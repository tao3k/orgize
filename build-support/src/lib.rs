//! Build-time helpers for generated `orgize` source artifacts.

mod org_aot_functions;
mod source_revision;

pub use org_aot_functions::write_org_aot_events;
pub use org_aot_functions::write_org_aot_functions;
pub use source_revision::write_source_revision;
