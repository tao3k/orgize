//! Parser-owned Org memory projections for agent workflows.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    Org,
    ast::{MemoryQuery, MemoryRecord, MemoryRecordState},
};

use super::{
    elements::collect_document_paths,
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
    let mut files = Vec::new();
    collect_document_paths(
        DocumentLanguage::Org,
        &memory_search_root(root, options),
        walk_config,
        &mut files,
    )?;
    files.sort();
    files.dedup();

    let query = MemoryQuery::new()
        .include_closed(options.include_closed)
        .include_archived(options.include_archived);
    let mut records = Vec::new();
    for path in files
        .into_iter()
        .filter(|path| file_matches_options(path, options))
    {
        let source =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        if options.plan_ledgers {
            if let Some(record) = plan_ledger_record_from_source(&path, &source, options) {
                records.push(record);
            }
            continue;
        }
        let document = Org::parse(&source).document();
        records.extend(
            document
                .memory_records(&query)
                .into_iter()
                .filter(|record| memory_record_matches_options(record, options))
                .map(|record| memory_record_projection(path.clone(), record)),
        );
    }
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
    });
    Ok(records)
}

fn plan_ledger_record_from_source(
    path: &Path,
    source: &str,
    options: &OrgMemorySearchOptions,
) -> Option<OrgMemorySearchRecord> {
    let first_line = source.lines().next()?.trim_end();
    let (todo, title, tags) = root_headline_parts(first_line)?;
    let state = if todo.as_deref() == Some("DONE") {
        MemoryRecordState::Closed
    } else {
        MemoryRecordState::Current
    };
    let properties = root_properties(source);
    let record = OrgMemorySearchRecord {
        path: path.to_path_buf(),
        start_line: 1,
        end_line: root_section_end_line(source),
        state,
        level: 1,
        title,
        todo,
        tags,
        properties,
        mtime: modified_seconds(path),
    };
    memory_search_record_matches_options(&record, options).then_some(record)
}

fn root_headline_parts(line: &str) -> Option<(Option<String>, String, Vec<String>)> {
    let rest = line.strip_prefix("* ")?;
    let (without_tags, tags) = split_headline_tags(rest);
    let (todo, title) = split_headline_todo(without_tags);
    Some((todo.map(str::to_string), title.to_string(), tags))
}

fn split_headline_tags(rest: &str) -> (&str, Vec<String>) {
    let trimmed = rest.trim_end();
    let Some((body, tag_tail)) = trimmed.rsplit_once(' ') else {
        return (trimmed, Vec::new());
    };
    if !tag_tail.starts_with(':') || !tag_tail.ends_with(':') {
        return (trimmed, Vec::new());
    }
    let tags = tag_tail
        .trim_matches(':')
        .split(':')
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect();
    (body.trim_end(), tags)
}

fn split_headline_todo(rest: &str) -> (Option<&str>, &str) {
    let Some((first, title)) = rest.trim_start().split_once(' ') else {
        return (None, rest.trim());
    };
    if first.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_') {
        (Some(first), title.trim())
    } else {
        (None, rest.trim())
    }
}

fn root_properties(source: &str) -> BTreeMap<String, String> {
    let mut in_drawer = false;
    let mut properties = BTreeMap::new();
    for line in source.lines().skip(1) {
        if !in_drawer && line.starts_with("** ") {
            break;
        }
        if line.trim() == ":PROPERTIES:" {
            in_drawer = true;
            continue;
        }
        if line.trim() == ":END:" {
            break;
        }
        if in_drawer && let Some((key, value)) = property_line(line) {
            properties.insert(key, value);
        }
    }
    properties
}

fn property_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(':')?;
    let (key, value) = rest.split_once(':')?;
    Some((key.trim().to_string(), value.trim().to_string()))
}

fn root_section_end_line(source: &str) -> usize {
    source
        .lines()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| line.starts_with("* ").then_some(index))
        .unwrap_or_else(|| source.lines().count().max(1))
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

fn memory_record_matches_options(record: &MemoryRecord, options: &OrgMemorySearchOptions) -> bool {
    (!options.root_only || record.source.start.line == 1)
        && memory_record_matches_scope(record, options)
        && memory_record_matches_terms(record, &options.terms)
        && memory_record_matches_contract(record, options.contract.as_deref())
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
        record
            .properties
            .get("PLAN_SESSION")
            .is_some_and(|value| value == expected)
    }) && options.plan.as_deref().is_none_or(|expected| {
        record
            .properties
            .get("PLAN_ID")
            .or_else(|| record.properties.get("ID"))
            .is_some_and(|value| value == expected)
    })
}

fn memory_search_record_matches_contract(
    record: &OrgMemorySearchRecord,
    contract: Option<&str>,
) -> bool {
    contract.is_none_or(|expected| {
        record
            .properties
            .get("CONTRACT_ORG")
            .is_some_and(|value| value == expected)
    })
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

fn memory_record_matches_scope(record: &MemoryRecord, options: &OrgMemorySearchOptions) -> bool {
    options
        .session
        .as_deref()
        .is_none_or(|expected| memory_record_property_eq(record, "PLAN_SESSION", expected))
        && options
            .plan
            .as_deref()
            .is_none_or(|expected| memory_record_property_eq(record, "PLAN_ID", expected))
}

fn memory_record_matches_contract(record: &MemoryRecord, contract: Option<&str>) -> bool {
    contract.is_none_or(|expected| memory_record_property_eq(record, "CONTRACT_ORG", expected))
}

fn memory_record_property_eq(record: &MemoryRecord, key: &str, expected: &str) -> bool {
    record
        .properties
        .iter()
        .any(|property| property.key.eq_ignore_ascii_case(key) && property.value == expected)
}

fn memory_record_matches_terms(record: &MemoryRecord, terms: &[String]) -> bool {
    terms.iter().all(|term| {
        let term = term.trim().to_ascii_lowercase();
        term.is_empty()
            || record.title.to_ascii_lowercase().contains(&term)
            || record
                .todo
                .as_ref()
                .is_some_and(|todo| todo.name.to_ascii_lowercase().contains(&term))
            || record.properties.iter().any(|property| {
                property.key.to_ascii_lowercase().contains(&term)
                    || property.value.to_ascii_lowercase().contains(&term)
            })
            || record.links.iter().any(|link| {
                link.path.to_ascii_lowercase().contains(&term)
                    || link.description.to_ascii_lowercase().contains(&term)
            })
    })
}

fn memory_record_projection(path: PathBuf, record: MemoryRecord) -> OrgMemorySearchRecord {
    OrgMemorySearchRecord {
        mtime: modified_seconds(&path),
        path,
        start_line: record.source.start.line,
        end_line: record.source.end.line,
        state: record.state,
        level: record.level,
        title: record.title,
        todo: record.todo.map(|todo| todo.name),
        tags: record.effective_tags,
        properties: record
            .properties
            .into_iter()
            .map(|property| (property.key, property.value))
            .collect(),
    }
}

fn modified_seconds(path: &Path) -> f64 {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default()
}
