//! Library-owned document element mapping and command surfaces.

mod command;
mod elements;
mod packets;
mod source_selection;

pub use command::{
    run_document_command, run_document_command_with_walk_config, run_md_command, run_org_command,
};
pub use elements::{
    DocumentElement, DocumentLanguage, DocumentWalkConfig, filter_elements, index_path,
    index_project, index_project_with_config,
};
pub use source_selection::{SourceLineRange, SourceSelector, select_source};

#[cfg(test)]
pub(crate) use command::compact_query_content;
