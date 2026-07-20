//! Document search, memory search, and status command handlers.

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::ast::MemoryRecordState;

use super::{
    command_query::{heading_facts, shell_arg},
    command_render::{
        print_fzf, print_fzf_toc, print_owner, print_prime, print_search_guide, print_toc,
    },
    elements::{
        display_path, escape_field, filter_elements, has_flag, index_path,
        index_project_with_config, last_existing_path, option_value, option_values,
    },
    memory_projection::{OrgMemorySearchOptions, OrgMemorySearchRecord, query_org_memory_records},
    model::{DocumentElement, DocumentLanguage, DocumentWalkConfig},
    packets::print_search_json,
};

pub(crate) fn run_search(
    language: DocumentLanguage,
    args: Vec<String>,
    walk_config: &DocumentWalkConfig,
) -> Result<ExitCode, String> {
    let json_output = has_flag(&args, "--json");
    let Some(view) = args.first().map(String::as_str) else {
        return Err(format!("{} search: expected view", language.id()));
    };

    match view {
        "guide" => {
            print_search_guide(language);
            Ok(ExitCode::SUCCESS)
        }
        "prime" => {
            let root = last_existing_path(&args[1..]).unwrap_or_else(|| PathBuf::from("."));
            let walk_config = walk_config_with_cli_excludes(walk_config, &args);
            let facts = index_project_with_config(language, &root, &walk_config)?;
            if json_output {
                print_search_json(language, "prime", &root, &facts, None)?;
            } else {
                print_prime(language, &root, &facts);
            }
            Ok(ExitCode::SUCCESS)
        }
        "toc" => {
            let root = last_existing_path(&args[1..]).unwrap_or_else(|| PathBuf::from("."));
            let walk_config = walk_config_with_cli_excludes(walk_config, &args);
            let facts = index_project_with_config(language, &root, &walk_config)?;
            let headings = heading_facts(&facts);
            if json_output {
                print_search_json(language, "toc", &root, &headings, None)?;
            } else {
                print_toc(language, &root, &headings);
            }
            Ok(ExitCode::SUCCESS)
        }
        "owner" => {
            let Some(owner) = args.get(1) else {
                return Err(format!("{} search owner: expected path", language.id()));
            };
            let path = PathBuf::from(owner);
            let facts = index_path(language, &path)?;
            if json_output {
                let root = path.parent().unwrap_or_else(|| Path::new("."));
                print_search_json(language, "owner", root, &facts, Some(owner))?;
            } else {
                print_owner(language, owner, &facts);
            }
            Ok(ExitCode::SUCCESS)
        }
        "fzf" => {
            let fzf_args = &args[1..];
            let root_arg_index = last_existing_path_arg_index(fzf_args);
            let terms = fzf_query_terms(fzf_args, root_arg_index);
            if terms.is_empty() {
                return Err(format!("{} search fzf: expected query", language.id()));
            };
            let root = root_arg_index
                .map(|index| PathBuf::from(&fzf_args[index]))
                .unwrap_or_else(|| PathBuf::from("."));
            let toc_output = fzf_toc_requested(fzf_args, root_arg_index);
            let walk_config = walk_config_with_cli_excludes(walk_config, &args);
            let facts = index_project_with_config(language, &root, &walk_config)?;
            if toc_output {
                let query = terms.join(" ");
                let headings = heading_facts_for_matching_documents(&facts, &terms);
                if json_output {
                    print_search_json(language, "fzf-toc", &root, &headings, Some(&query))?;
                } else {
                    print_fzf_toc(language, &query, &root, &headings);
                }
            } else {
                let query = terms
                    .first()
                    .expect("terms is non-empty after earlier check")
                    .as_str();
                let matches = filter_elements(&facts, query);
                if json_output {
                    print_search_json(language, "fzf", &root, &matches, Some(query))?;
                } else {
                    print_fzf(language, query, &root, &matches);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        "memory" => run_memory_search(language, &args[1..], walk_config),
        view => Err(format!(
            "{} search: unsupported view `{view}`",
            language.id()
        )),
    }
}
pub(super) fn walk_config_with_cli_excludes(
    walk_config: &DocumentWalkConfig,
    args: &[String],
) -> DocumentWalkConfig {
    let mut walk_config = walk_config.clone();
    for dir in option_values(args, "--exclude-dir") {
        if !dir.trim().is_empty() && !walk_config.ignore_dirs.iter().any(|item| item == &dir) {
            walk_config.ignore_dirs.push(dir);
        }
    }
    walk_config
}

fn run_memory_search(
    language: DocumentLanguage,
    args: &[String],
    walk_config: &DocumentWalkConfig,
) -> Result<ExitCode, String> {
    if language != DocumentLanguage::Org {
        return Err(format!(
            "{} search memory: Org memory projection is only supported for Org documents",
            language.id()
        ));
    }
    let root = last_existing_path(args).unwrap_or_else(|| PathBuf::from("."));
    let walk_config = walk_config_with_cli_excludes(walk_config, args);
    let explicit_session = option_value(args, "--session");
    let current_session = explicit_session
        .is_none()
        .then(current_agent_session_id)
        .flatten();
    let session = explicit_session.or(current_session.as_deref());
    let plan = option_value(args, "--plan");
    let terms = option_values(args, "--term");
    let options = memory_search_options(args, session, plan, &terms);
    let records = query_org_memory_records(&root, &walk_config, &options)?;
    print_memory_search(
        language,
        &root,
        &options,
        memory_search_limit(args),
        &records,
    );
    Ok(ExitCode::SUCCESS)
}

fn memory_search_options(
    args: &[String],
    session: Option<&str>,
    plan: Option<&str>,
    terms: &[String],
) -> OrgMemorySearchOptions {
    let mut options = if has_flag(args, "--plan-ledgers") {
        OrgMemorySearchOptions::plan_ledgers()
    } else {
        OrgMemorySearchOptions::default()
    };
    options.session = session.map(str::to_string);
    options.plan = plan.map(str::to_string);
    options.terms = terms.to_vec();
    options.include_closed = has_flag(args, "--include-closed") || has_flag(args, "--include-done");
    options.include_archived = has_flag(args, "--include-archived");
    if let Some(contract) = option_value(args, "--contract") {
        options.contract = Some(contract.to_string());
    }
    if let Some(file_prefix) = option_value(args, "--file-prefix") {
        options.file_prefix = Some(file_prefix.to_string());
    }
    if has_flag(args, "--root-only") {
        options.root_only = true;
    }
    options
}

fn current_agent_session_id() -> Option<String> {
    env_value("CODEX_THREAD_ID")
        .or_else(|| env_value("CLAUDE_CODE_SESSION_ID"))
        .or_else(|| env_value("CLAUDE_CODE_REMOTE_SESSION_ID"))
        .or_else(|| env_value("AGENT_SESSION_ID"))
        .or_else(|| env_value("SESSION_ID"))
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().and_then(non_empty_value)
}

fn non_empty_value(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn print_memory_search(
    language: DocumentLanguage,
    root: &Path,
    options: &OrgMemorySearchOptions,
    limit: usize,
    records: &[OrgMemorySearchRecord],
) {
    println!(
        "[search-memory] lang={} root={} current={} shown={} session={} plan={} terms={} planLedgers={}",
        language.id(),
        display_path(root),
        records.len(),
        records.len().min(limit),
        options.session.as_deref().unwrap_or("-"),
        options.plan.as_deref().unwrap_or("-"),
        if options.terms.is_empty() {
            "-".to_string()
        } else {
            options.terms.join(",")
        },
        options.plan_ledgers
    );
    for record in records.iter().take(limit) {
        println!("{}", render_memory_record(record));
    }
    println!(
        "|next current-session=asp org search memory --session <SESSION_ID> --workspace {} --view seeds",
        display_path(root)
    );
    if let Some(session) = &options.session {
        let intent = if options.terms.is_empty() {
            "unfinished org task".to_string()
        } else {
            options.terms.join(" ")
        };
        let mut command = format!(
            "asp-memory-engine recall-plan --state .data/omni-memory/state.json --intent {} --session {}",
            shell_arg(&intent),
            shell_arg(session)
        );
        if let Some(plan) = &options.plan {
            command.push_str(" --plan ");
            command.push_str(&shell_arg(plan));
        }
        println!("|python-memory-engine next={}", command);
    }
}

fn memory_search_limit(args: &[String]) -> usize {
    option_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(12)
}

fn render_memory_record(record: &OrgMemorySearchRecord) -> String {
    let todo = record.todo.as_deref().unwrap_or("-");
    let session = record
        .properties
        .get("SESSION_ID")
        .map(String::as_str)
        .unwrap_or("-");
    let plan = record
        .properties
        .get("PLAN_ID")
        .map(String::as_str)
        .unwrap_or("-");
    format!(
        "|memory {}:{}-{} state={} todo=\"{}\" session=\"{}\" plan=\"{}\" title=\"{}\"",
        display_path(&record.path),
        record.start_line,
        record.end_line,
        memory_record_state(record.state),
        escape_field(todo),
        escape_field(session),
        escape_field(plan),
        escape_field(&record.title),
    )
}

fn memory_record_state(state: MemoryRecordState) -> &'static str {
    match state {
        MemoryRecordState::Current => "current",
        MemoryRecordState::Closed => "closed",
        MemoryRecordState::Archived => "archived",
        MemoryRecordState::Background => "background",
    }
}
fn last_existing_path_arg_index(args: &[String]) -> Option<usize> {
    args.iter()
        .enumerate()
        .rev()
        .filter(|(_, arg)| !arg.starts_with('-'))
        .find_map(|(index, arg)| PathBuf::from(arg).exists().then_some(index))
}

fn fzf_toc_requested(args: &[String], root_arg_index: Option<usize>) -> bool {
    option_value(args, "--view") == Some("toc")
        || args
            .iter()
            .enumerate()
            .any(|(index, arg)| index > 0 && Some(index) != root_arg_index && arg == "toc")
}

fn fzf_query_terms(args: &[String], root_arg_index: Option<usize>) -> Vec<String> {
    let mut terms = Vec::new();
    let mut skip_next = false;
    for (index, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if Some(index) == root_arg_index {
            continue;
        }
        if arg == "--view" {
            skip_next = true;
            continue;
        }
        if arg.starts_with("--") || (index > 0 && arg == "toc") {
            continue;
        }
        terms.push(arg.clone());
    }
    terms
}

fn heading_facts_for_matching_documents(
    facts: &[DocumentElement],
    terms: &[String],
) -> Vec<DocumentElement> {
    let paths = facts
        .iter()
        .map(|fact| fact.path.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let matching_paths = paths
        .into_iter()
        .filter(|path| {
            let document_facts = facts
                .iter()
                .filter(|candidate| candidate.path.as_str() == *path)
                .collect::<Vec<_>>();
            terms
                .iter()
                .all(|term| document_facts.iter().any(|fact| fact.matches(term)))
        })
        .collect::<std::collections::BTreeSet<_>>();
    facts
        .iter()
        .filter(|fact| fact.kind == "heading" && matching_paths.contains(fact.path.as_str()))
        .cloned()
        .collect()
}
