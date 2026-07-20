//! CLI command routing for `asp org` and `asp md` document providers.

use std::process::ExitCode;

use super::{
    command_query::run_query,
    command_render::{print_guide, run_elements_query},
    command_search::run_search,
    model::{DocumentLanguage, DocumentWalkConfig},
};

pub fn run_org_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Org, args)
}

/// Run an `asp md` document command.
pub fn run_md_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Markdown, args)
}

/// Route a document command using the default project walk policy.
pub fn run_document_command(
    language: DocumentLanguage,
    args: Vec<String>,
) -> Result<ExitCode, String> {
    run_document_command_with_walk_config(language, args, DocumentWalkConfig::default())
}

/// Route a document command using caller-provided project walk policy.
pub fn run_document_command_with_walk_config(
    language: DocumentLanguage,
    args: Vec<String>,
    walk_config: DocumentWalkConfig,
) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_guide(language);
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "guide" => {
            print_guide(language);
            Ok(ExitCode::SUCCESS)
        }
        "search" => run_search(language, args.collect(), &walk_config),
        "query" => run_query(language, args.collect(), &walk_config),
        "elements-query" => run_elements_query(language, args.collect()),
        "-h" | "--help" | "help" => {
            print_guide(language);
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!(
            "{}: unsupported document command `{command}`",
            language.id()
        )),
    }
}
