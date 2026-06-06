//! Agent-facing document command API.

pub use crate::document::DocumentLanguage;

/// Run a document command with the same arguments accepted by the `asp org` or
/// `asp md` document facade after the language id.
pub fn run_document_command(language: DocumentLanguage, args: Vec<String>) -> Result<(), String> {
    crate::document::run_document_command(language, args).map(|_| ())
}

/// Run an Org document command.
pub fn run_org_command(args: Vec<String>) -> Result<(), String> {
    run_document_command(DocumentLanguage::Org, args)
}

/// Run a Markdown document command.
pub fn run_md_command(args: Vec<String>) -> Result<(), String> {
    run_document_command(DocumentLanguage::Markdown, args)
}
