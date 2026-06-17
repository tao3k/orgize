//! Agent-facing document command API.

use std::process::ExitCode;

pub use crate::document::{DocumentLanguage, DocumentWalkConfig};

/// Run a document command with the same arguments accepted by the `asp org` or
/// `asp md` document facade after the language id.
pub fn run_document_command(language: DocumentLanguage, args: Vec<String>) -> Result<(), String> {
    crate::document::run_document_command(language, args).map(|_| ())
}

/// Run a document command with explicit project-walk configuration supplied by
/// the embedding ASP facade.
pub fn run_document_command_with_walk_config(
    language: DocumentLanguage,
    args: Vec<String>,
    walk_config: DocumentWalkConfig,
) -> Result<(), String> {
    crate::document::run_document_command_with_walk_config(language, args, walk_config).map(|_| ())
}

/// Run an Org document command.
pub fn run_org_command(args: Vec<String>) -> Result<(), String> {
    run_document_command(DocumentLanguage::Org, args)
}

/// Run an Org contract command, such as `contract trace`.
pub fn run_org_contract_command(args: Vec<String>) -> Result<(), String> {
    org_cli_exit_result("org contract", crate::cli::org_contract_trace::run(args)?)
}

/// Run an Orgize CLI command that is embedded by ASP's `asp org` facade.
pub fn run_org_cli_command(args: Vec<String>) -> Result<(), String> {
    org_cli_exit_result("orgize", crate::cli::run_args(args)?)
}

/// Run a Markdown document command.
pub fn run_md_command(args: Vec<String>) -> Result<(), String> {
    run_document_command(DocumentLanguage::Markdown, args)
}

fn org_cli_exit_result(label: &str, code: ExitCode) -> Result<(), String> {
    if code == ExitCode::SUCCESS {
        Ok(())
    } else {
        Err(format!("{label} exited with status {code:?}"))
    }
}
