//! CLI command routing for `orgize org` and `orgize md` document providers.

use std::process::ExitCode;

use super::{
    command_query::run_query,
    command_render::{print_guide, run_elements_query},
    model::{DocumentLanguage, DocumentWalkConfig},
};

pub fn run_org_command(args: Vec<String>) -> Result<ExitCode, String> {
    run_document_command(DocumentLanguage::Org, args)
}

/// Run an `orgize md` document command.
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
        "capture" if language == DocumentLanguage::Org => run_org_capture(args.collect()),
        "contract" if language == DocumentLanguage::Org => {
            crate::cli::org_contract_trace::run(args.collect())
        }
        "guide" => {
            print_guide(language);
            Ok(ExitCode::SUCCESS)
        }
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

fn run_org_capture(args: Vec<String>) -> Result<ExitCode, String> {
    match crate::ast::org_capture_plan_command(args)? {
        crate::ast::OrgCapturePlanCommandOutput::Help(usage) => {
            eprintln!("{usage}");
            Ok(ExitCode::SUCCESS)
        }
        crate::ast::OrgCapturePlanCommandOutput::Plan(plan) => {
            print!("{plan}");
            Ok(ExitCode::SUCCESS)
        }
    }
}
