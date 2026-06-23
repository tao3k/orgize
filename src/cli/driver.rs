//! Command-line interface implementation for the `orgize` binary.

use std::{
    env, fs,
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::{
    Org,
    ast::{
        AgendaDate, AgendaQuery, AgentPlanningQuery, OrgCapturePlanCommandOutput, PriorityProfile,
        PriorityValue, PropertySchemaContract, PropertySchemaField, PropertySchemaRegistry,
        PropertySchemaValueRule, SddNodeRecord, SparseTreeQuery,
    },
    fmt::{FormatOptions, format_org},
    lint::{LintOptions, lint_org_with_options},
};

/// Runs the command-line interface with process arguments and stdio.
pub fn run_from_env() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("orgize: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    run_args(env::args().skip(1).collect())
}

pub(crate) fn run_args(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        print_usage();
        return Ok(ExitCode::from(2));
    };

    match command.as_str() {
        "agent-planning" => run_agent_planning(args.collect()),
        "capture-plan" => run_capture_plan(args.collect()),
        "contract" => super::org_contract_trace::run(args.collect()),
        "eval" => super::eval::run(args.collect()),
        "export" => run_export(args.collect()),
        "fmt" => run_fmt(args.collect()),
        "elements-query" | "guide" | "search" | "query" => {
            let mut command_args = vec![command.to_string()];
            command_args.extend(args);
            crate::document::run_org_command(command_args)
        }
        "lint" => run_lint(args.collect()),
        "md" | "markdown" => crate::document::run_md_command(args.collect()),
        "sdd" => run_sdd(args.collect()),
        "sparse-tree" => run_sparse_tree(args.collect()),
        "task-list" => run_task_list(args.collect()),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        command => Err(format!("unknown command `{command}`")),
    }
}

fn run_capture_plan(args: Vec<String>) -> Result<ExitCode, String> {
    match crate::ast::org_capture_plan_command(args)? {
        OrgCapturePlanCommandOutput::Help(usage) => {
            eprintln!("{usage}");
            Ok(ExitCode::SUCCESS)
        }
        OrgCapturePlanCommandOutput::Plan(plan) => {
            print!("{plan}");
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn run_export(args: Vec<String>) -> Result<ExitCode, String> {
    let mut args = args.into_iter();
    let Some(format) = args.next() else {
        print_export_usage();
        return Ok(ExitCode::from(2));
    };

    match format.as_str() {
        "md" | "markdown" => run_export_markdown(args.collect()),
        "-h" | "--help" | "help" => {
            print_export_usage();
            Ok(ExitCode::SUCCESS)
        }
        format => Err(format!("unknown export format `{format}`")),
    }
}

fn run_export_markdown(paths: Vec<String>) -> Result<ExitCode, String> {
    if paths.is_empty() {
        let source = read_stdin()?;
        print!("{}", Org::parse(source).to_markdown());
        return Ok(ExitCode::SUCCESS);
    }

    for path in collect_org_paths(&paths)? {
        let source = fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
        print!("{}", Org::parse(source).to_markdown());
    }

    Ok(ExitCode::SUCCESS)
}

fn run_sdd(args: Vec<String>) -> Result<ExitCode, String> {
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

struct OrgSource {
    display_path: String,
    source: String,
}

fn collect_org_sources(paths: &[String]) -> Result<Vec<OrgSource>, String> {
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

fn run_agent_planning(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_agent_planning_args(args)?;
    if args.help {
        print_agent_planning_usage();
        return Ok(ExitCode::SUCCESS);
    }
    let start = parse_required_agenda_date(args.date.as_deref(), "agent-planning --date")?;
    let end = args
        .end
        .as_deref()
        .map(|value| parse_agenda_date(value, "agent-planning --end"))
        .transpose()?
        .unwrap_or(start);

    for source in collect_org_sources(&args.paths)? {
        let document = Org::parse(&source.source).document();
        let mut agenda_query = AgendaQuery::new(start, end)
            .include_done(args.include_done)
            .include_archived(args.include_archived)
            .include_comments(args.include_comments);
        if let Some(expression) = &args.match_expression {
            agenda_query = agenda_query
                .match_expression(expression)
                .map_err(|error| error.to_string())?;
        }
        let query = AgentPlanningQuery::new(agenda_query);
        print!(
            "{}",
            document
                .agent_planning_snapshot(&query)
                .to_compact_text(&source.display_path)
        );
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Default)]
struct AgentPlanningArgs {
    help: bool,
    date: Option<String>,
    end: Option<String>,
    include_done: bool,
    include_archived: bool,
    include_comments: bool,
    match_expression: Option<String>,
    paths: Vec<String>,
}

fn parse_agent_planning_args(args: Vec<String>) -> Result<AgentPlanningArgs, String> {
    let mut parsed = AgentPlanningArgs::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-h" | "--help" => parsed.help = true,
            "--date" => {
                index += 1;
                parsed.date = Some(required_flag_value(&args, index, "--date")?.to_string());
            }
            "--end" => {
                index += 1;
                parsed.end = Some(required_flag_value(&args, index, "--end")?.to_string());
            }
            "--include-done" => parsed.include_done = true,
            "--include-archived" => parsed.include_archived = true,
            "--include-comments" => parsed.include_comments = true,
            "--match" => {
                index += 1;
                parsed.match_expression =
                    Some(required_flag_value(&args, index, "--match")?.to_string());
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown agent-planning flag `{arg}`"));
            }
            _ => parsed.paths.push(arg.clone()),
        }
        index += 1;
    }
    Ok(parsed)
}

fn run_sparse_tree(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_sparse_tree_args(args)?;
    if args.help {
        print_sparse_tree_usage();
        return Ok(ExitCode::SUCCESS);
    }

    for source in collect_org_sources(&args.paths)? {
        let document = Org::parse(&source.source).document();
        let mut query = SparseTreeQuery::new()
            .include_done(!args.exclude_done)
            .include_archived(!args.exclude_archived)
            .include_comments(args.include_comments)
            .explain_skips(args.explain_skips)
            .source_file(source.display_path.clone());
        if let Some(expression) = &args.match_expression {
            query = query
                .match_expression(expression)
                .map_err(|error| error.to_string())?;
        }
        if let Some(text) = &args.text {
            query = query.text(text);
        }
        print!(
            "{}",
            document
                .sparse_tree_projection(&query)
                .to_compact_text(&source.display_path)
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn run_task_list(args: Vec<String>) -> Result<ExitCode, String> {
    let args = parse_task_list_args(args)?;
    if args.help {
        print_task_list_usage();
        return Ok(ExitCode::SUCCESS);
    }
    let _cache_requested = args.cached;

    let mut rendered = String::new();
    let mut remaining = args.limit;
    for source in collect_org_sources(&args.paths)? {
        if remaining == 0 {
            break;
        }
        let document = Org::parse(&source.source).document();
        let query = SparseTreeQuery::new()
            .include_done(true)
            .include_archived(true)
            .source_file(source.display_path.clone());
        let projection = document.sparse_tree_projection(&query);
        let cards = projection
            .cards
            .iter()
            .filter(|card| task_list_card_matches(card, &args))
            .take(remaining)
            .collect::<Vec<_>>();
        remaining = remaining.saturating_sub(cards.len());
        rendered.push_str(&render_task_list_cards(&source.display_path, &cards));
    }

    if rendered.is_empty() {
        println!("[ok] orgize task-list");
    } else {
        print!("{rendered}");
    }
    Ok(ExitCode::SUCCESS)
}

#[derive(Default)]
struct TaskListArgs {
    help: bool,
    cached: bool,
    view: TaskListView,
    text: Option<String>,
    tags: Vec<String>,
    include_done: bool,
    include_archived: bool,
    limit: usize,
    paths: Vec<String>,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum TaskListView {
    #[default]
    Active,
    Done,
    Archived,
    Achievement,
    ArchiveCandidate,
    ClosureNeeded,
    Repeating,
}

fn parse_task_list_args(args: Vec<String>) -> Result<TaskListArgs, String> {
    let mut parsed = TaskListArgs {
        limit: 20,
        ..TaskListArgs::default()
    };
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-h" | "--help" => parsed.help = true,
            "--cached" => parsed.cached = true,
            "--view" => {
                index += 1;
                parsed.view = parse_task_list_view(required_flag_value(&args, index, "--view")?)?;
            }
            "--text" => {
                index += 1;
                parsed.text = Some(required_flag_value(&args, index, "--text")?.to_string());
            }
            "--tag" => {
                index += 1;
                parsed
                    .tags
                    .push(required_flag_value(&args, index, "--tag")?.to_string());
            }
            "--include-done" => parsed.include_done = true,
            "--include-archived" => parsed.include_archived = true,
            "--limit" => {
                index += 1;
                parsed.limit = required_flag_value(&args, index, "--limit")?
                    .parse::<usize>()
                    .map_err(|_| "task-list --limit requires a non-negative integer".to_string())?;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown task-list flag `{arg}`"));
            }
            _ => parsed.paths.push(arg.clone()),
        }
        index += 1;
    }
    Ok(parsed)
}

fn parse_task_list_view(value: &str) -> Result<TaskListView, String> {
    match value {
        "active" => Ok(TaskListView::Active),
        "done" => Ok(TaskListView::Done),
        "archived" => Ok(TaskListView::Archived),
        "achievement" => Ok(TaskListView::Achievement),
        "archive-candidate" => Ok(TaskListView::ArchiveCandidate),
        "closure-needed" => Ok(TaskListView::ClosureNeeded),
        "repeating" => Ok(TaskListView::Repeating),
        _ => Err(format!("unsupported task-list view `{value}`")),
    }
}

fn task_list_card_matches(card: &crate::ast::SparseTreeCard, args: &TaskListArgs) -> bool {
    let Some(todo) = &card.todo else {
        return false;
    };
    if !args.tags.iter().all(|tag| {
        card.effective_tags
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(tag))
    }) {
        return false;
    }
    if let Some(text) = &args.text
        && !task_card_contains_text(card, text)
    {
        return false;
    }

    let done = matches!(todo.state, crate::ast::TodoState::Done);
    match args.view {
        TaskListView::Active => {
            (!done || args.include_done) && (!card.archive.archived || args.include_archived)
        }
        TaskListView::Done => done && (!card.archive.archived || args.include_archived),
        TaskListView::Archived => card.archive.archived,
        TaskListView::Achievement => card
            .effective_tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("achievement")),
        TaskListView::ArchiveCandidate => done && !card.archive.archived,
        TaskListView::ClosureNeeded => !done && title_has_complete_cookie(&card.title),
        TaskListView::Repeating => task_card_has_repeater(card),
    }
}

fn task_card_contains_text(card: &crate::ast::SparseTreeCard, text: &str) -> bool {
    let needle = text.to_ascii_lowercase();
    card.title.to_ascii_lowercase().contains(&needle)
        || card
            .outline_path
            .iter()
            .any(|part| part.to_ascii_lowercase().contains(&needle))
        || card
            .preview
            .as_ref()
            .is_some_and(|preview| preview.to_ascii_lowercase().contains(&needle))
        || card.properties.iter().any(|property| {
            property.key.to_ascii_lowercase().contains(&needle)
                || property.value.to_ascii_lowercase().contains(&needle)
        })
}

fn title_has_complete_cookie(title: &str) -> bool {
    title
        .split_whitespace()
        .any(|part| part == "[100%]" || complete_fraction_cookie(part))
}

fn complete_fraction_cookie(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let Some((done, total)) = inner.split_once('/') else {
        return false;
    };
    let Ok(done) = done.parse::<usize>() else {
        return false;
    };
    let Ok(total) = total.parse::<usize>() else {
        return false;
    };
    total > 0 && done == total
}

fn task_card_has_repeater(card: &crate::ast::SparseTreeCard) -> bool {
    task_timestamp_has_repeater(card.planning.scheduled.as_ref())
        || task_timestamp_has_repeater(card.planning.deadline.as_ref())
}

fn task_timestamp_has_repeater(timestamp: Option<&crate::ast::Timestamp>) -> bool {
    timestamp.is_some_and(|timestamp| timestamp.raw.contains("+") || timestamp.raw.contains(".+"))
}

fn render_task_list_cards(path: &str, cards: &[&crate::ast::SparseTreeCard]) -> String {
    if cards.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("[TASK_LIST] ");
    output.push_str(path);
    output.push('\n');
    output.push_str("rows: ");
    output.push_str(&cards.len().to_string());
    output.push('\n');
    for card in cards {
        output.push_str("- ");
        if let Some(todo) = &card.todo {
            output.push_str(&todo.name);
            output.push(' ');
        }
        output.push_str(&card.title);
        output.push('\n');
        output.push_str("  @ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&card.source.start.line.to_string());
        output.push(':');
        output.push_str(&card.source.start.column.to_string());
        output.push('\n');
        output.push_str("  outline: ");
        output.push_str(&card.outline_path.join(" / "));
        output.push('\n');
        if !card.effective_tags.is_empty() {
            output.push_str("  tags: ");
            output.push_str(&card.effective_tags.join(":"));
            output.push('\n');
        }
        push_task_timestamp(&mut output, "scheduled", card.planning.scheduled.as_ref());
        push_task_timestamp(&mut output, "deadline", card.planning.deadline.as_ref());
        push_task_timestamp(&mut output, "closed", card.planning.closed.as_ref());
    }
    output
}

fn push_task_timestamp(
    output: &mut String,
    label: &str,
    timestamp: Option<&crate::ast::Timestamp>,
) {
    if let Some(timestamp) = timestamp {
        output.push_str("  ");
        output.push_str(label);
        output.push_str(": ");
        output.push_str(&timestamp.raw);
        output.push('\n');
    }
}

#[derive(Default)]
struct SparseTreeArgs {
    help: bool,
    text: Option<String>,
    match_expression: Option<String>,
    exclude_done: bool,
    exclude_archived: bool,
    include_comments: bool,
    explain_skips: bool,
    paths: Vec<String>,
}

fn parse_sparse_tree_args(args: Vec<String>) -> Result<SparseTreeArgs, String> {
    let mut parsed = SparseTreeArgs::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "-h" | "--help" => parsed.help = true,
            "--text" => {
                index += 1;
                parsed.text = Some(required_flag_value(&args, index, "--text")?.to_string());
            }
            "--match" => {
                index += 1;
                parsed.match_expression =
                    Some(required_flag_value(&args, index, "--match")?.to_string());
            }
            "--exclude-done" => parsed.exclude_done = true,
            "--exclude-archived" => parsed.exclude_archived = true,
            "--include-comments" => parsed.include_comments = true,
            "--explain-skips" => parsed.explain_skips = true,
            _ if arg.starts_with('-') => {
                return Err(format!("unknown sparse-tree flag `{arg}`"));
            }
            _ => parsed.paths.push(arg.clone()),
        }
        index += 1;
    }
    Ok(parsed)
}

fn required_flag_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &'static str,
) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_required_agenda_date(
    value: Option<&str>,
    label: &'static str,
) -> Result<AgendaDate, String> {
    parse_agenda_date(
        value.ok_or_else(|| format!("{label} requires YYYY-MM-DD"))?,
        label,
    )
}

fn parse_agenda_date(value: &str, label: &'static str) -> Result<AgendaDate, String> {
    AgendaDate::parse_ymd(value).ok_or_else(|| format!("{label} expects YYYY-MM-DD, got `{value}`"))
}

fn run_fmt(args: Vec<String>) -> Result<ExitCode, String> {
    let mut check = false;
    let mut paths = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--write" | "-w" => {}
            "-h" | "--help" => {
                print_fmt_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown fmt flag `{arg}`")),
            _ => paths.push(arg),
        }
    }

    let options = FormatOptions::default();
    let mut changed = false;

    if paths.is_empty() {
        let source = read_stdin()?;
        let formatted = format_org(&source, &options);
        changed |= formatted.changed;
        if check {
            if formatted.changed {
                eprintln!("<stdin>: needs formatting");
            }
        } else {
            print!("{}", formatted.output);
        }
    } else {
        for path in collect_org_paths(&paths)? {
            let display_path = display_path(&path);
            let source =
                fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
            let formatted = format_org(&source, &options);
            changed |= formatted.changed;
            if check {
                if formatted.changed {
                    eprintln!("{display_path}: needs formatting");
                }
            } else {
                if formatted.changed {
                    fs::write(&path, formatted.output)
                        .map_err(|error| format_path_error(&path, error))?;
                }
            }
        }
    }

    Ok(if check && changed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_lint(args: Vec<String>) -> Result<ExitCode, String> {
    let mut output_format = LintOutputFormat::Compact;
    let mut priority_highest = None;
    let mut priority_lowest = None;
    let mut priority_default = None;
    let mut property_schema_registry_paths = Vec::new();
    let mut org_contract_registry_paths = Vec::new();
    let mut fix = false;
    let mut paths = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--format" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --format requires `compact`, `text`, or `json`".to_string());
                };
                output_format = LintOutputFormat::parse(value)?;
            }
            "--json" => output_format = LintOutputFormat::Json,
            "--priority-highest" => {
                index += 1;
                priority_highest = Some(parse_priority_flag(&args, index, "--priority-highest")?);
            }
            "--priority-lowest" => {
                index += 1;
                priority_lowest = Some(parse_priority_flag(&args, index, "--priority-lowest")?);
            }
            "--priority-default" => {
                index += 1;
                priority_default = Some(parse_priority_flag(&args, index, "--priority-default")?);
            }
            "--property-schema-registry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --property-schema-registry requires a JSON path".to_string());
                };
                property_schema_registry_paths.push(PathBuf::from(value));
            }
            "--org-contract-registry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("lint --org-contract-registry requires an Org path".to_string());
                };
                org_contract_registry_paths.push(PathBuf::from(value));
            }
            "--fix" => fix = true,
            "-h" | "--help" => {
                print_lint_usage();
                return Ok(ExitCode::SUCCESS);
            }
            _ if arg.starts_with('-') => return Err(format!("unknown lint flag `{arg}`")),
            _ => paths.push(arg.clone()),
        }
        index += 1;
    }

    let priority_profile =
        priority_profile_from_flags(priority_highest, priority_lowest, priority_default)?;
    let property_schema_registry =
        load_property_schema_registries(&property_schema_registry_paths)?;
    let org_contract_registry =
        super::org_contract_registry::load_org_contract_registries(&org_contract_registry_paths)?;
    let base_lint_options = LintOptions {
        priority_profile,
        property_schema_registry,
        org_contract_registry,
        ..LintOptions::default()
    };

    let mut reports = Vec::new();
    if paths.is_empty() {
        if fix {
            return Err("lint --fix requires at least one Org file or directory path".to_string());
        }
        let source = read_stdin()?;
        let report = lint_org_with_options(&source, &base_lint_options);
        reports.push(LintFileReport {
            path: "<stdin>".to_string(),
            source,
            report,
        });
    } else {
        for path in collect_org_paths(&paths)? {
            let display_path = display_path(&path);
            let mut source =
                fs::read_to_string(&path).map_err(|error| format_path_error(&path, error))?;
            if fix {
                let formatted = format_org(&source, &FormatOptions::default());
                if formatted.changed {
                    fs::write(&path, &formatted.output)
                        .map_err(|error| format_path_error(&path, error))?;
                    source = formatted.output;
                }
            }
            let lint_options = LintOptions {
                source_path: Some(path.clone()),
                include_base_dir: path.parent().map(Path::to_path_buf),
                attachment_base_dir: path.parent().map(Path::to_path_buf),
                file_base_dir: path.parent().map(Path::to_path_buf),
                ..base_lint_options.clone()
            };
            let report = lint_org_with_options(&source, &lint_options);
            reports.push(LintFileReport {
                path: display_path,
                source,
                report,
            });
        }
    }

    let has_findings = reports.iter().any(|file| !file.report.is_clean());
    match output_format {
        LintOutputFormat::Compact => {
            print!("{}", render_lint_compact(&reports));
        }
        LintOutputFormat::Text => {
            for file in &reports {
                print!("{}", file.report.to_text(&file.path));
            }
        }
        LintOutputFormat::Json => {
            print!("{{\"files\":[");
            for (index, file) in reports.iter().enumerate() {
                if index > 0 {
                    print!(",");
                }
                print!("{}", file.report.to_json_file(&file.path));
            }
            println!("]}}");
        }
    }

    Ok(if has_findings {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    Ok(input)
}

struct LintFileReport {
    path: String,
    source: String,
    report: crate::lint::LintReport,
}

fn render_lint_compact(reports: &[LintFileReport]) -> String {
    let rendered = reports
        .iter()
        .filter(|file| !file.report.is_clean())
        .map(|file| file.report.to_compact_text(&file.path, &file.source))
        .collect::<Vec<_>>();

    if rendered.is_empty() {
        "[ok] orgize lint\n".to_string()
    } else {
        rendered.join("\n")
    }
}

#[derive(Clone, Copy)]
enum LintOutputFormat {
    Compact,
    Text,
    Json,
}

impl LintOutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "compact" => Ok(Self::Compact),
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("unsupported lint output format `{value}`")),
        }
    }
}

fn parse_priority_flag(
    args: &[String],
    index: usize,
    flag: &'static str,
) -> Result<PriorityValue, String> {
    let Some(value) = args.get(index) else {
        return Err(format!("lint {flag} requires a priority value"));
    };
    PriorityValue::parse(value).ok_or_else(|| format!("unsupported priority value `{value}`"))
}

fn priority_profile_from_flags(
    highest: Option<PriorityValue>,
    lowest: Option<PriorityValue>,
    default: Option<PriorityValue>,
) -> Result<PriorityProfile, String> {
    if highest.is_none() && lowest.is_none() && default.is_none() {
        return Ok(PriorityProfile::org_default());
    }
    let profile = PriorityProfile::org_default();
    let highest = highest.unwrap_or_else(|| profile.highest().clone());
    let lowest = lowest.unwrap_or_else(|| profile.lowest().clone());
    let default = default.unwrap_or_else(|| profile.default_priority().clone());
    PriorityProfile::new(highest, lowest, default).ok_or_else(|| {
        "priority profile must use one priority family and satisfy highest <= default <= lowest"
            .to_string()
    })
}

fn load_property_schema_registries(paths: &[PathBuf]) -> Result<PropertySchemaRegistry, String> {
    let mut registry = PropertySchemaRegistry::default();
    for path in paths {
        let loaded = load_property_schema_registry(path)?;
        registry.contracts.extend(loaded.contracts);
    }
    Ok(registry)
}

fn load_property_schema_registry(path: &Path) -> Result<PropertySchemaRegistry, String> {
    let source = fs::read_to_string(path).map_err(|error| format_path_error(path, error))?;
    let value = serde_json::from_str::<serde_json::Value>(&source)
        .map_err(|error| format!("{}: invalid JSON: {error}", display_path(path)))?;
    let mut registry = property_schema_registry_from_json(&value)
        .map_err(|error| format!("{}: {error}", display_path(path)))?;
    add_property_schema_file_aliases(&mut registry, path);
    Ok(registry)
}

fn property_schema_registry_from_json(
    value: &serde_json::Value,
) -> Result<PropertySchemaRegistry, String> {
    if let Some(contracts) = value.get("contracts") {
        return Ok(PropertySchemaRegistry::new(parse_contracts(contracts)?));
    }
    if value.get("id").is_some() {
        return Ok(PropertySchemaRegistry::new([parse_schema_contract(value)?]));
    }
    if value.is_array() {
        return Ok(PropertySchemaRegistry::new(parse_contracts(value)?));
    }
    Err(
        "expected a registry object with `contracts`, a contract object, or a contract array"
            .to_string(),
    )
}

fn parse_contracts(value: &serde_json::Value) -> Result<Vec<PropertySchemaContract>, String> {
    value
        .as_array()
        .ok_or_else(|| "`contracts` must be an array".to_string())?
        .iter()
        .enumerate()
        .map(|(index, contract)| {
            parse_schema_contract(contract).map_err(|error| format!("contracts[{index}]: {error}"))
        })
        .collect()
}

fn parse_schema_contract(value: &serde_json::Value) -> Result<PropertySchemaContract, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "contract must be an object".to_string())?;
    let id = json_string(object, "id")
        .ok_or_else(|| "contract requires string `id`".to_string())?
        .to_string();
    let allow_unknown_properties =
        json_optional_bool(object, "allowUnknownProperties", "allow_unknown_properties")?
            .unwrap_or(true);
    let mut contract =
        PropertySchemaContract::new(id).allow_unknown_properties(allow_unknown_properties);

    if let Some(aliases) = object.get("aliases") {
        for alias in json_string_array(aliases, "aliases")? {
            contract = contract.alias(alias);
        }
    }

    if let Some(fields) = object.get("fields") {
        for (index, field) in fields
            .as_array()
            .ok_or_else(|| "`fields` must be an array".to_string())?
            .iter()
            .enumerate()
        {
            contract = contract.field(
                parse_schema_field(field).map_err(|error| format!("fields[{index}]: {error}"))?,
            );
        }
    }

    Ok(contract)
}

fn parse_schema_field(value: &serde_json::Value) -> Result<PropertySchemaField, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "field must be an object".to_string())?;
    let key = json_string(object, "key")
        .ok_or_else(|| "field requires string `key`".to_string())?
        .to_string();
    let required = json_optional_bool(object, "required", "required")?.unwrap_or(false);
    let value_rule = json_field(object, "valueRule", "value_rule")
        .map(parse_schema_value_rule)
        .transpose()?
        .unwrap_or(PropertySchemaValueRule::Any);

    Ok(if required {
        PropertySchemaField::required(key, value_rule)
    } else {
        PropertySchemaField::optional(key, value_rule)
    })
}

fn parse_schema_value_rule(value: &serde_json::Value) -> Result<PropertySchemaValueRule, String> {
    if let Some(kind) = value.as_str() {
        return match kind {
            "any" => Ok(PropertySchemaValueRule::Any),
            "nonEmpty" => Ok(PropertySchemaValueRule::NonEmpty),
            "oneOf" => Err("valueRule `oneOf` requires an object with `values`".to_string()),
            _ => Err(format!("unsupported valueRule kind `{kind}`")),
        };
    }
    let object = value
        .as_object()
        .ok_or_else(|| "valueRule must be an object or string".to_string())?;
    let kind = json_string(object, "kind")
        .ok_or_else(|| "valueRule requires string `kind`".to_string())?;
    match kind {
        "any" => Ok(PropertySchemaValueRule::Any),
        "nonEmpty" => Ok(PropertySchemaValueRule::NonEmpty),
        "oneOf" => {
            let values = object
                .get("values")
                .ok_or_else(|| "valueRule `oneOf` requires `values`".to_string())?;
            Ok(PropertySchemaValueRule::OneOf(json_string_array(
                values, "values",
            )?))
        }
        _ => Err(format!("unsupported valueRule kind `{kind}`")),
    }
}

fn json_field<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    camel: &str,
    snake: &str,
) -> Option<&'a serde_json::Value> {
    object.get(camel).or_else(|| object.get(snake))
}

fn json_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    object.get(key).and_then(serde_json::Value::as_str)
}

fn json_optional_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    camel: &str,
    snake: &str,
) -> Result<Option<bool>, String> {
    json_field(object, camel, snake)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| format!("`{camel}` must be a boolean"))
        })
        .transpose()
}

fn json_string_array(value: &serde_json::Value, label: &str) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("`{label}` must be an array"))?
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("`{label}` item {index} must be a string"))
        })
        .collect()
}

fn add_property_schema_file_aliases(registry: &mut PropertySchemaRegistry, path: &Path) {
    let bases = property_schema_file_alias_bases(path);
    let single_contract_id =
        (registry.contracts.len() == 1).then(|| registry.contracts[0].id.clone());
    for contract in &mut registry.contracts {
        for base in &bases {
            push_property_schema_alias(contract, format!("{base}#{}", contract.id));
            push_property_schema_alias(contract, format!("file:{base}#{}", contract.id));
        }
        if single_contract_id.as_deref() == Some(contract.id.as_str()) {
            for base in &bases {
                push_property_schema_alias(contract, base.clone());
                push_property_schema_alias(contract, format!("file:{base}"));
            }
        }
    }
}

fn property_schema_file_alias_bases(path: &Path) -> Vec<String> {
    let mut bases = Vec::new();
    push_property_schema_alias_base(&mut bases, path);
    if let Ok(canonical) = path.canonicalize() {
        push_property_schema_alias_base(&mut bases, canonical.as_path());
    }
    bases
}

fn push_property_schema_alias_base(bases: &mut Vec<String>, path: &Path) {
    let value = normalize_property_schema_path(path);
    if !value.is_empty() && !bases.iter().any(|base| base == &value) {
        bases.push(value.clone());
    }
    match value.strip_prefix("./") {
        Some(stripped) if !stripped.is_empty() && !bases.iter().any(|base| base == stripped) => {
            bases.push(stripped.to_string());
        }
        _ => {}
    }
}

fn normalize_property_schema_path(path: &Path) -> String {
    display_path(path)
}

fn push_property_schema_alias(contract: &mut PropertySchemaContract, alias: String) {
    if alias.trim().is_empty()
        || alias == contract.id
        || contract.aliases.iter().any(|existing| existing == &alias)
    {
        return;
    }
    contract.aliases.push(alias);
}

fn print_usage() {
    eprintln!(
        "Usage: orgize <agent-planning|capture-plan|contract|elements-query|eval|export|fmt|guide|lint|md|query|search|sdd|sparse-tree|task-list> [options] [PATH ...]"
    );
}

fn print_export_usage() {
    eprintln!("Usage: orgize export <md|markdown> [PATH ...]");
}

fn print_fmt_usage() {
    eprintln!("Usage: orgize fmt [--check] [PATH ...]");
}

fn print_lint_usage() {
    eprintln!(
        "Usage: orgize lint [--fix] [--format compact|text|json] [--priority-highest VALUE] [--priority-default VALUE] [--priority-lowest VALUE] [--property-schema-registry PATH.json] [--property-schema-registry PATH.json ...] [--org-contract-registry PATH.org] [--org-contract-registry PATH.org ...] [PATH ...]"
    );
}

fn print_sdd_usage() {
    eprintln!("Usage: orgize sdd <status|graph-diff> [options] [PATH ...]");
}

fn print_sdd_status_usage() {
    eprintln!("Usage: orgize sdd status [--json] [--issues-only] [--fail-on-issues] [PATH ...]");
}

fn print_sdd_graph_diff_usage() {
    eprintln!("Usage: orgize sdd graph-diff [--fail-on-drift] [PATH ...]");
}

fn print_agent_planning_usage() {
    eprintln!(
        "Usage: orgize agent-planning --date YYYY-MM-DD [--end YYYY-MM-DD] [--include-done] [--include-archived] [--include-comments] [--match EXPR] [PATH ...]"
    );
}

fn print_sparse_tree_usage() {
    eprintln!(
        "Usage: orgize sparse-tree [--text TEXT] [--match EXPR] [--exclude-done] [--exclude-archived] [--include-comments] [--explain-skips] [PATH ...]"
    );
}

fn print_task_list_usage() {
    eprintln!(
        "Usage: orgize task-list [--cached] [--view active|done|archived|achievement|archive-candidate|closure-needed|repeating] [--text TEXT] [--tag TAG] [--include-done] [--include-archived] [--limit N] [PATH ...]"
    );
}

fn collect_org_paths(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        collect_org_path(Path::new(path), &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_org_path(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format_path_error(path, error))?;
    if metadata.is_file() {
        if !is_org_file(path) {
            return Err(format!("{}: expected .org file", display_path(path)));
        }
        files.push(path.to_path_buf());
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", display_path(path)));
    }

    let mut entries = fs::read_dir(path)
        .map_err(|error| format_path_error(path, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format_path_error(path, error))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        let entry_type = entry
            .file_type()
            .map_err(|error| format_path_error(&entry_path, error))?;
        if entry_type.is_dir() {
            collect_org_path(&entry_path, files)?;
        } else if entry_type.is_file() && is_org_file(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn is_org_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("org"))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn format_path_error(path: &Path, error: std::io::Error) -> String {
    format!("{}: {}", display_path(path), stable_io_error(&error))
}

fn stable_io_error(error: &std::io::Error) -> String {
    match error.kind() {
        ErrorKind::NotFound => "No such file or directory (os error 2)".to_string(),
        _ => error.to_string(),
    }
}
