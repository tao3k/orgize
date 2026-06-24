//! Library-owned document element mapping and command surfaces.

mod command;
mod elements;
mod line_index;
mod markdown_elements;
mod memory_projection;
mod model;
mod org_elements;
mod packets;
mod source_selection;

pub use command::{
    run_document_command, run_document_command_with_walk_config, run_md_command, run_org_command,
};
pub use elements::{filter_elements, index_path, index_project, index_project_with_config};
pub use memory_projection::{
    OrgMemorySearchOptions, OrgMemorySearchRecord, query_org_memory_records,
};
pub use model::{DocumentElement, DocumentLanguage, DocumentWalkConfig};
pub use source_selection::{SourceLineRange, SourceSelector, select_source};

#[cfg(test)]
pub(crate) use command::compact_query_content;

#[cfg(test)]
#[path = "../../tests/unit/document_line_index.rs"]
mod line_index_tests;
