//! Agent-facing document command API.

/// Supported document languages for the agent document provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentLanguage {
    /// Org-mode documents parsed by orgize.
    Org,
    /// Markdown documents parsed through comrak-backed document elements.
    Markdown,
}

/// Run a document command with the same arguments accepted by the `orgize`
/// binary after its optional `md` prefix.
pub fn run_document_command(language: DocumentLanguage, args: Vec<String>) -> Result<(), String> {
    match language {
        DocumentLanguage::Org => crate::cli::document::run_org_command(args),
        DocumentLanguage::Markdown => crate::cli::document::run_md_command(args),
    }
    .map(|_| ())
}

/// Run an Org document command.
pub fn run_org_command(args: Vec<String>) -> Result<(), String> {
    run_document_command(DocumentLanguage::Org, args)
}

/// Run a Markdown document command.
pub fn run_md_command(args: Vec<String>) -> Result<(), String> {
    run_document_command(DocumentLanguage::Markdown, args)
}
