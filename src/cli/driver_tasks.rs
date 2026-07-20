//! Agent planning, sparse-tree, and task-list command handlers.

use std::process::ExitCode;

use crate::ast::{AgendaDate, AgentPlanningQuery, SparseTreeQuery};

use super::driver_usage::{
    print_agent_planning_usage, print_sparse_tree_usage, print_task_list_usage,
};

pub(crate) fn run_agent_planning(args: Vec<String>) -> Result<ExitCode, String> {
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

pub(crate) fn run_sparse_tree(args: Vec<String>) -> Result<ExitCode, String> {
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

pub(crate) fn run_task_list(args: Vec<String>) -> Result<ExitCode, String> {
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
use super::driver_sdd::collect_org_sources;
use crate::ast::AgendaQuery;
use crate::org::Org;
