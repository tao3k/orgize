//! Parser-owned Org memory projections for agent workflows.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    ast::MemoryRecordState,
    org_aot::{OrgAotDocument, parse_org_aot},
};

use crate::document::{
    elements::collect_document_paths,
    line_index::LineIndex,
    model::{DocumentLanguage, DocumentWalkConfig},
};

use super::plan_ledger::query_plan_ledger_records;

#[derive(Clone, Debug, Default)]
pub struct OrgMemorySearchOptions {
    pub session: Option<String>,
    pub plan: Option<String>,
    pub terms: Vec<String>,
    pub contract: Option<String>,
    pub file_prefix: Option<String>,
    pub root_only: bool,
    pub include_closed: bool,
    pub include_archived: bool,
    pub plan_ledgers: bool,
}

#[derive(Clone, Debug)]
pub struct OrgMemorySearchRecord {
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub state: MemoryRecordState,
    pub level: usize,
    pub title: String,
    pub todo: Option<String>,
    pub tags: Vec<String>,
    pub properties: BTreeMap<String, String>,
    pub mtime: f64,
}

impl OrgMemorySearchOptions {
    pub fn plan_ledgers() -> Self {
        Self {
            contract: Some("agent.plan.v1".to_string()),
            file_prefix: Some("agent-plan-".to_string()),
            root_only: true,
            plan_ledgers: true,
            ..Self::default()
        }
    }
}

pub fn query_org_memory_records(
    root: &Path,
    walk_config: &DocumentWalkConfig,
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    if options.plan_ledgers {
        return query_plan_ledger_records(root, walk_config, options);
    }

    let mut files = Vec::new();
    collect_document_paths(
        DocumentLanguage::Org,
        &memory_search_root(root, options),
        walk_config,
        &mut files,
    )?;
    files.sort();
    files.dedup();

    let mut records = Vec::new();
    for path in files
        .into_iter()
        .filter(|path| file_matches_options(path, options))
    {
        let source =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let document = parse_org_aot(&source)
            .map_err(|error| format!("{}: Org AOT parse: {error:?}", path.display()))?;
        records.extend(project_memory_records(&document, &source, &path, options)?);
    }
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
    });
    Ok(records)
}

fn project_memory_records(
    document: &OrgAotDocument,
    source: &str,
    path: &Path,
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    let lines = LineIndex::new(source);
    let mtime = modified_seconds(path);
    let mut selected = Vec::new();
    for headline in document.headlines() {
        let Some(title) = headline.display_title() else {
            return Err(format!("{}: headline lacks an AOT title", path.display()));
        };
        if headline.is_comment() {
            continue;
        }

        let tags = headline.effective_tags();
        let properties = headline
            .properties()
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect();
        let mut closed = false;
        let mut planned = false;
        for (key, _) in headline.planning() {
            if key.eq_ignore_ascii_case("CLOSED") {
                closed = true;
            } else if key.eq_ignore_ascii_case("SCHEDULED") || key.eq_ignore_ascii_case("DEADLINE")
            {
                planned = true;
            }
        }
        let archived = tags.iter().any(|tag| tag.eq_ignore_ascii_case("ARCHIVE"));
        let state = match document.headline_memory_state(headline.id(), closed, planned, archived) {
            "archived" => MemoryRecordState::Archived,
            "closed" => MemoryRecordState::Closed,
            "current" => MemoryRecordState::Current,
            _ => MemoryRecordState::Background,
        };
        let start = usize::from(headline.range().start());
        let end = usize::from(headline.range().end()).saturating_sub(1);
        let record = OrgMemorySearchRecord {
            path: path.to_path_buf(),
            start_line: lines.line_for(start),
            end_line: lines.line_for(end),
            state,
            level: headline.level(),
            title,
            todo: headline.todo_keyword(),
            tags,
            properties,
            mtime,
        };
        if (!options.root_only || record.start_line == 1)
            && (options.include_closed || record.state != MemoryRecordState::Closed)
            && (options.include_archived || record.state != MemoryRecordState::Archived)
            && memory_search_record_matches_scope(&record, options)
            && memory_search_record_matches_contract(&record, options.contract.as_deref())
            && memory_record_terms_match(document, headline.id(), &record, &options.terms)
        {
            selected.push(record);
        }
    }
    Ok(selected)
}

fn memory_record_terms_match(
    document: &OrgAotDocument,
    headline_id: usize,
    record: &OrgMemorySearchRecord,
    terms: &[String],
) -> bool {
    if terms.is_empty() {
        return true;
    }
    let mut pending = terms
        .iter()
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| {
            !term.is_empty()
                && !record.title.to_ascii_lowercase().contains(term)
                && !record
                    .todo
                    .as_ref()
                    .is_some_and(|todo| todo.to_ascii_lowercase().contains(term))
                && !record.properties.iter().any(|(key, value)| {
                    key.to_ascii_lowercase().contains(term)
                        || value.to_ascii_lowercase().contains(term)
                })
        })
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return true;
    }
    let records = document.records();
    let end = document
        .graph_index()
        .subtree_end(headline_id)
        .unwrap_or(headline_id + 1);
    let mut cursor = headline_id + 1;
    while cursor < end {
        let node = &records[cursor];
        if node.kind == "headline" {
            cursor = document
                .graph_index()
                .subtree_end(cursor)
                .unwrap_or(cursor + 1);
            continue;
        }
        if node.kind == "link" {
            pending.retain(|term| {
                !["path", "description"].into_iter().any(|field| {
                    node.field(field)
                        .is_some_and(|value| value.to_ascii_lowercase().contains(term))
                })
            });
            if pending.is_empty() {
                return true;
            }
        }
        cursor += 1;
    }
    false
}

pub(super) fn memory_search_root(root: &Path, options: &OrgMemorySearchOptions) -> PathBuf {
    if options.plan_ledgers {
        let plans_root = root.join("flow").join("plans");
        if plans_root.is_dir() {
            return plans_root;
        }
    }
    root.to_path_buf()
}

pub(super) fn file_matches_options(path: &Path, options: &OrgMemorySearchOptions) -> bool {
    options.file_prefix.as_ref().is_none_or(|prefix| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(prefix))
    })
}

pub(super) fn memory_search_record_matches_options(
    record: &OrgMemorySearchRecord,
    options: &OrgMemorySearchOptions,
) -> bool {
    (!options.root_only || record.start_line == 1)
        && memory_search_record_matches_scope(record, options)
        && memory_search_record_matches_terms(record, &options.terms)
        && memory_search_record_matches_contract(record, options.contract.as_deref())
        && (options.include_closed || record.state != MemoryRecordState::Closed)
        && (options.include_archived || record.state != MemoryRecordState::Archived)
}

fn memory_search_record_matches_scope(
    record: &OrgMemorySearchRecord,
    options: &OrgMemorySearchOptions,
) -> bool {
    options.session.as_deref().is_none_or(|expected| {
        memory_property(record, "SESSION_ID").is_some_and(|value| value == expected)
    }) && options.plan.as_deref().is_none_or(|expected| {
        memory_property(record, "PLAN_ID")
            .or_else(|| memory_property(record, "ID"))
            .is_some_and(|value| value == expected)
    })
}

fn memory_search_record_matches_contract(
    record: &OrgMemorySearchRecord,
    contract: Option<&str>,
) -> bool {
    contract.is_none_or(|expected| {
        memory_property(record, "CONTRACT_ORG").is_some_and(|value| value == expected)
    })
}

fn memory_property<'a>(record: &'a OrgMemorySearchRecord, key: &str) -> Option<&'a str> {
    record
        .properties
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.as_str())
}

fn memory_search_record_matches_terms(record: &OrgMemorySearchRecord, terms: &[String]) -> bool {
    terms.iter().all(|term| {
        let term = term.trim().to_ascii_lowercase();
        term.is_empty()
            || record.title.to_ascii_lowercase().contains(&term)
            || record
                .todo
                .as_ref()
                .is_some_and(|todo| todo.to_ascii_lowercase().contains(&term))
            || record
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains(&term))
            || record.properties.iter().any(|(key, value)| {
                key.to_ascii_lowercase().contains(&term)
                    || value.to_ascii_lowercase().contains(&term)
            })
    })
}

fn modified_seconds(path: &Path) -> f64 {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|mtime| modified_seconds_from_time(Some(mtime)))
        .unwrap_or_default()
}

fn modified_seconds_from_time(modified: Option<std::time::SystemTime>) -> Option<f64> {
    modified
        .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs_f64())
}
