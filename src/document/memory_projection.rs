//! Parser-owned Org memory projections for agent workflows.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    thread,
    time::UNIX_EPOCH,
};

use crate::{
    ast::MemoryRecordState,
    org_aot::{OrgAotDocument, parse_org_aot},
};

use super::{
    elements::collect_document_paths,
    line_index::LineIndex,
    model::{DocumentLanguage, DocumentWalkConfig},
};

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

fn query_plan_ledger_records(
    root: &Path,
    walk_config: &DocumentWalkConfig,
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    let started = std::time::Instant::now();
    let root = memory_search_root(root, options);
    let mut files = Vec::new();
    collect_plan_ledger_paths(&root, walk_config, options, &mut files)?;
    let walk_elapsed = started.elapsed();
    let mut records = plan_ledger_records_from_paths(&files, options)?;
    let projection_elapsed = started.elapsed() - walk_elapsed;
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
    });
    if std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER").is_some() {
        eprintln!(
            "plan ledger stages: files={} workers={} walk={walk_elapsed:?} projection={projection_elapsed:?} total={:?}",
            files.len(),
            plan_ledger_worker_count(files.len()),
            started.elapsed()
        );
    }
    Ok(records)
}

fn collect_plan_ledger_paths(
    root: &Path,
    walk_config: &DocumentWalkConfig,
    options: &OrgMemorySearchOptions,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let metadata = fs::metadata(root).map_err(|error| format!("{}: {error}", root.display()))?;
    if metadata.is_file() {
        if DocumentLanguage::Org.matches_path(root) && file_matches_options(root, options) {
            files.push(root.to_path_buf());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", root.display()));
    }

    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if file_type.is_dir() {
            if should_skip_plan_ledger_directory(name, walk_config) {
                continue;
            }
            collect_plan_ledger_paths(&path, walk_config, options, files)?;
        } else if file_type.is_file()
            && DocumentLanguage::Org.matches_path(&path)
            && file_matches_options(&path, options)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn plan_ledger_records_from_paths(
    paths: &[PathBuf],
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    if paths.len() < 64 {
        let mut records = Vec::new();
        for path in paths {
            collect_plan_ledger_file(path, options, &mut records)?;
        }
        return Ok(records);
    }

    // Reading independent files can overlap while the Scheme-AOT graph for
    // each file is built. Bound workers by the work available, not a fixed
    // machine-wide cap, and keep each worker's chunk large enough to amortize
    // thread startup.
    let worker_count = plan_ledger_worker_count(paths.len());
    let chunk_size = paths.len().div_ceil(worker_count);
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in paths.chunks(chunk_size) {
            handles.push(scope.spawn(move || {
                let mut records = Vec::new();
                for path in chunk {
                    collect_plan_ledger_file(path, options, &mut records)?;
                }
                Ok::<_, String>(records)
            }));
        }

        let mut records = Vec::new();
        for handle in handles {
            records.extend(
                handle
                    .join()
                    .map_err(|_| "plan ledger worker panicked".to_string())??,
            );
        }
        Ok(records)
    })
}

fn plan_ledger_worker_count(path_count: usize) -> usize {
    if path_count < 64 {
        return 1;
    }
    thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .saturating_mul(8)
        .min(path_count.div_ceil(64))
}

fn collect_plan_ledger_file(
    path: &Path,
    options: &OrgMemorySearchOptions,
    records: &mut Vec<OrgMemorySearchRecord>,
) -> Result<(), String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if let Some(record) = plan_ledger_record_from_source(path, &source, options)? {
        records.push(record);
    }
    Ok(())
}

fn should_skip_plan_ledger_directory(name: &str, walk_config: &DocumentWalkConfig) -> bool {
    if walk_config
        .include_hidden_dirs
        .iter()
        .any(|included| included == name)
    {
        return false;
    }
    walk_config
        .ignore_dirs
        .iter()
        .any(|ignored| ignored == name)
        || (name.starts_with('.') && !name.starts_with(".org"))
}

fn plan_ledger_record_from_source(
    path: &Path,
    source: &str,
    options: &OrgMemorySearchOptions,
) -> Result<Option<OrgMemorySearchRecord>, String> {
    let document = parse_org_aot(source)
        .map_err(|error| format!("{}: Org AOT parse: {error:?}", path.display()))?;
    let records = document.records();
    let Some(headline) = records.iter().find(|record| {
        record.kind == "headline"
            && record.parent_id == Some(0)
            && record.field("markers") == Some("*")
            && usize::from(record.range.start()) == 0
    }) else {
        return Ok(None);
    };
    let todo = document.headline_todo_keyword(headline.id);
    let title = document
        .headline_display_title(headline.id)
        .ok_or_else(|| format!("{}: headline lacks an AOT title", path.display()))?;
    let tags = headline.values("tag").map(str::to_owned).collect();
    let state = if document.headline_todo_type(headline.id) == Some("done") {
        MemoryRecordState::Closed
    } else {
        MemoryRecordState::Current
    };
    let mut properties = BTreeMap::new();
    for drawer in headline
        .child_ids
        .iter()
        .filter_map(|id| records.get(*id))
        .filter(|record| record.kind == "property-drawer")
    {
        for property in drawer
            .child_ids
            .iter()
            .filter_map(|id| records.get(*id))
            .filter(|record| record.kind == "node-property")
        {
            if let (Some(key), Some(value)) = (property.field("key"), property.field("value")) {
                properties.insert(key.to_string(), value.to_string());
            }
        }
    }
    let end_line =
        LineIndex::new(source).line_for(usize::from(headline.range.end()).saturating_sub(1));
    let record = OrgMemorySearchRecord {
        path: path.to_path_buf(),
        start_line: 1,
        end_line,
        state,
        level: 1,
        title,
        todo,
        tags,
        properties,
        mtime: 0.0,
    };
    Ok(memory_search_record_matches_options(&record, options).then_some(record))
}

fn memory_search_root(root: &Path, options: &OrgMemorySearchOptions) -> PathBuf {
    if options.plan_ledgers {
        let plans_root = root.join("flow").join("plans");
        if plans_root.is_dir() {
            return plans_root;
        }
    }
    root.to_path_buf()
}

fn file_matches_options(path: &Path, options: &OrgMemorySearchOptions) -> bool {
    options.file_prefix.as_ref().is_none_or(|prefix| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(prefix))
    })
}

fn memory_search_record_matches_options(
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
