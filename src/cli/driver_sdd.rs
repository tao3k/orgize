//! SDD command handlers and renderers for the orgize CLI.

use std::{fs, process::ExitCode};

use super::driver_paths::{collect_org_paths, display_path, format_path_error, read_stdin};
use crate::{
    ast::SddNodeRecord,
    lint::{lint_model::LintOptions, lint_org_with_options},
    org::Org,
};

use super::driver_usage::{print_sdd_graph_diff_usage, print_sdd_status_usage, print_sdd_usage};

pub(crate) fn run_sdd(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_sdd_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "status" => run_sdd_status(args.collect()),
        "graph-diff" => run_sdd_graph_diff(args.collect()),
        "-h" | "--help" | "help" => {
            print_sdd_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown sdd command `{command}`")),
    }
}

fn run_sdd_status(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_sdd_status_args(args)?;
    if args.help {
        print_sdd_status_usage();
        return Ok(ExitCode::SUCCESS);
    }

    let files = collect_sdd_status_files(&args.paths, args.issues_only)?;
    let issue_count = files.iter().map(|file| file.issue_count).sum::<usize>();
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schemaVersion": 1,
                "files": files.iter().map(sdd_status_file_to_json).collect::<Vec<_>>(),
            }))
            .expect("sdd status JSON should serialize")
        );
    } else if files.is_empty() {
        println!("[ok] orgize sdd status: no SDD issues");
    } else {
        for file in &files {
            print!("{}", sdd_status_text(file));
        }
    }

    Ok(if args.fail_on_issues && issue_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

#[derive(Default)]
struct SddStatusArgs {
    help: bool,
    json: bool,
    issues_only: bool,
    fail_on_issues: bool,
    paths: Vec<String>,
}

fn parse_sdd_status_args(args: Vec<String>) -> Result<SddStatusArgs, String> {
    args.into_iter()
        .try_fold(SddStatusArgs::default(), |mut parsed, arg| {
            match arg.as_str() {
                "-h" | "--help" => parsed.help = true,
                "--json" => parsed.json = true,
                "--issues-only" => parsed.issues_only = true,
                "--fail-on-issues" => parsed.fail_on_issues = true,
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown sdd status flag `{arg}`"));
                }
                _ => parsed.paths.push(arg),
            }
            Ok(parsed)
        })
}

fn run_sdd_graph_diff(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_sdd_graph_diff_args(args)?;
    if args.help {
        print_sdd_graph_diff_usage();
        return Ok(ExitCode::SUCCESS);
    }

    let files = collect_sdd_graph_diff_files(&args.paths)?;
    let drift_count = files.iter().map(|file| file.drifts.len()).sum::<usize>();
    if drift_count == 0 {
        println!("[ok] orgize sdd graph-diff");
    } else {
        for file in &files {
            if file.drifts.is_empty() {
                continue;
            }
            print!("{}", sdd_graph_diff_text(file));
        }
    }

    Ok(if args.fail_on_drift && drift_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

#[derive(Default)]
struct SddGraphDiffArgs {
    help: bool,
    fail_on_drift: bool,
    paths: Vec<String>,
}

fn parse_sdd_graph_diff_args(args: Vec<String>) -> Result<SddGraphDiffArgs, String> {
    args.into_iter()
        .try_fold(SddGraphDiffArgs::default(), |mut parsed, arg| {
            match arg.as_str() {
                "-h" | "--help" => parsed.help = true,
                "--fail-on-drift" => parsed.fail_on_drift = true,
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown sdd graph-diff flag `{arg}`"));
                }
                _ => parsed.paths.push(arg),
            }
            Ok(parsed)
        })
}

pub(super) struct OrgSource {
    pub(super) display_path: String,
    pub(super) source: String,
}

pub(super) fn collect_org_sources(paths: &[String]) -> Result<Vec<OrgSource>, String> {
    if paths.is_empty() {
        return Ok(vec![OrgSource {
            display_path: "<stdin>".to_string(),
            source: read_stdin()?,
        }]);
    }

    collect_org_paths(paths)?
        .into_iter()
        .map(|path| {
            let display_path = display_path(&path);
            let source =
                fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
            Ok(OrgSource {
                display_path,
                source,
            })
        })
        .collect()
}

struct SddStatusFile {
    path: String,
    records: Vec<SddNodeRecord>,
    issue_count: usize,
}

fn collect_sdd_status_files(
    paths: &[String],
    issues_only: bool,
) -> Result<Vec<SddStatusFile>, String> {
    Ok(collect_org_sources(paths)?
        .into_iter()
        .filter_map(|source| {
            let document = Org::parse(&source.source).document();
            let issue_count = count_sdd_issues(&source.source);
            (!issues_only || issue_count > 0).then_some(SddStatusFile {
                path: source.display_path,
                records: document.sdd_node_records(),
                issue_count,
            })
        })
        .collect::<Vec<_>>())
}

fn sdd_status_text(file: &SddStatusFile) -> String {
    let mut output = crate::ast::SddStatus {
        records: file.records.clone(),
    }
    .to_compact_text(&file.path);
    if file.issue_count > 0 {
        output.push_str("issues: ");
        output.push_str(&file.issue_count.to_string());
        output.push('\n');
    }
    output
}

fn sdd_status_file_to_json(file: &SddStatusFile) -> serde_json::Value {
    serde_json::json!({
        "path": file.path,
        "issueCount": file.issue_count,
        "records": file.records.iter().map(sdd_record_to_json).collect::<Vec<_>>(),
    })
}

fn sdd_record_to_json(record: &SddNodeRecord) -> serde_json::Value {
    serde_json::json!({
        "title": &record.title,
        "kind": record.kind.as_str(),
        "id": record.id.as_deref(),
        "parent": record.parent.as_ref().map(|parent| serde_json::json!({
            "raw": &parent.raw,
            "targetId": parent.target_id.as_deref(),
            "label": parent.label.as_deref(),
        })),
        "status": record.status.as_ref().map(|status| status.as_str()),
        "outlinePath": &record.outline_path,
        "source": {
            "line": record.source.start.line,
            "column": record.source.start.column,
            "rangeStart": record.source.range_start,
            "rangeEnd": record.source.range_end,
        },
        "capability": record.capability.as_deref(),
        "viewpoint": record.viewpoint.as_deref(),
        "concern": record.concern.as_deref(),
        "quality": record.quality.as_deref(),
        "rationale": record.rationale.as_deref(),
        "slug": record.slug.as_deref(),
    })
}

fn count_sdd_issues(source: &str) -> usize {
    lint_org_with_options(source, &LintOptions::default())
        .findings
        .iter()
        .filter(|finding| is_sdd_lint_code(finding.code))
        .count()
}

fn is_sdd_lint_code(code: &str) -> bool {
    matches!(
        code,
        "ORG031" | "ORG032" | "ORG033" | "ORG034" | "ORG035" | "ORG036" | "ORG037"
    )
}

struct SddGraphDiffFile {
    path: String,
    drifts: Vec<SddGraphDrift>,
}

struct SddGraphDrift {
    title: String,
    line: usize,
    column: usize,
    semantic_parent: Option<String>,
    outline_parent: Option<String>,
    outline_parent_title: Option<String>,
}

fn collect_sdd_graph_diff_files(paths: &[String]) -> Result<Vec<SddGraphDiffFile>, String> {
    collect_org_sources(paths)?
        .into_iter()
        .map(|source| {
            let document = Org::parse(&source.source).document();
            let records = document.sdd_node_records();
            Ok(SddGraphDiffFile {
                path: source.display_path,
                drifts: sdd_graph_drifts(&records),
            })
        })
        .collect()
}

fn sdd_graph_drifts(records: &[SddNodeRecord]) -> Vec<SddGraphDrift> {
    records
        .iter()
        .filter_map(|record| {
            let semantic_parent_id = record
                .parent
                .as_ref()
                .and_then(|parent| parent.target_id.as_deref());
            let semantic_parent_display = record.parent.as_ref().map(|parent| {
                parent
                    .target_id
                    .as_deref()
                    .unwrap_or(parent.raw.as_str())
                    .to_string()
            });
            let outline_parent = nearest_sdd_outline_parent(records, record);
            let outline_parent_id = outline_parent.and_then(|parent| parent.id.as_deref());
            if semantic_parent_id == outline_parent_id {
                return None;
            }

            Some(SddGraphDrift {
                title: record.title.clone(),
                line: record.source.start.line,
                column: record.source.start.column,
                semantic_parent: semantic_parent_display,
                outline_parent: outline_parent_id.map(str::to_string),
                outline_parent_title: outline_parent.map(|parent| parent.title.clone()),
            })
        })
        .collect()
}

fn nearest_sdd_outline_parent<'a>(
    records: &'a [SddNodeRecord],
    record: &SddNodeRecord,
) -> Option<&'a SddNodeRecord> {
    records
        .iter()
        .filter(|candidate| {
            candidate.outline_path.len() < record.outline_path.len()
                && record.outline_path.starts_with(&candidate.outline_path)
        })
        .max_by_key(|candidate| candidate.outline_path.len())
}

fn sdd_graph_diff_text(file: &SddGraphDiffFile) -> String {
    let mut output = String::new();
    output.push_str("[SDD_GRAPH_DRIFT] ");
    output.push_str(&file.path);
    output.push('\n');
    for drift in &file.drifts {
        output.push_str("- ");
        output.push_str(&drift.title);
        output.push_str(" @ ");
        output.push_str(&drift.line.to_string());
        output.push(':');
        output.push_str(&drift.column.to_string());
        output.push('\n');
        output.push_str("  semantic-parent: ");
        output.push_str(drift.semantic_parent.as_deref().unwrap_or("<none>"));
        output.push('\n');
        output.push_str("  outline-parent: ");
        output.push_str(drift.outline_parent.as_deref().unwrap_or("<none>"));
        if let Some(title) = &drift.outline_parent_title {
            output.push_str(" (");
            output.push_str(title);
            output.push(')');
        }
        output.push('\n');
    }
    output
}
