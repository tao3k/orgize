//! Bounded host I/O and handoff; all Org syntax comes from the native parser.
use super::records::{
    file_matches_options, memory_search_record_matches_options, memory_search_root,
};
use super::{OrgMemorySearchOptions, OrgMemorySearchRecord};
use crate::document::elements::{bounded_org_batch_len, load_sources};
use crate::org_aot::{OrgAotDocument, parse_org_aot};
use crate::{
    ast::MemoryRecordState,
    document::{
        line_index::LineIndex,
        model::{DocumentLanguage, DocumentWalkConfig},
    },
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    thread,
};

pub(super) fn query_plan_ledger_records(
    root: &Path,
    walk_config: &DocumentWalkConfig,
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    let started = std::time::Instant::now();
    let root = memory_search_root(root, options);
    let mut files = Vec::new();
    collect_plan_ledger_paths(&root, walk_config, options, &mut files)?;
    let walk_elapsed = started.elapsed();
    let work = plan_ledger_records_from_paths(&files, options)?;
    let mut records = work.records;
    let projection_elapsed = started.elapsed() - walk_elapsed;
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
    });
    if std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER").is_some()
        || std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER_STAGES").is_some()
    {
        #[cfg(feature = "runtime-profile")]
        if std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER_STAGES").is_some() {
            eprintln!(
                "plan ledger worker sums: diagnostic only, overlapping stages, ns except native count metrics; {:?}",
                work.timings
            );
        }
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

struct PlanLedgerWork {
    records: Vec<OrgMemorySearchRecord>,
    #[cfg(feature = "runtime-profile")]
    timings: BTreeMap<&'static str, u64>,
}

impl PlanLedgerWork {
    fn new(records: Vec<OrgMemorySearchRecord>) -> Self {
        Self {
            records,
            #[cfg(feature = "runtime-profile")]
            timings: BTreeMap::new(),
        }
    }
}

fn observe_plan_ledger_files(
    paths: &[PathBuf],
    options: &OrgMemorySearchOptions,
) -> Result<PlanLedgerWork, String> {
    #[cfg(feature = "runtime-profile")]
    if std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER_STAGES").is_some()
        && !crate::runtime_profile::is_active()
    {
        let (records, timings) =
            crate::runtime_profile::measure(|| collect_plan_ledger_files(paths, options));
        return records.map(|records| PlanLedgerWork { records, timings });
    }
    collect_plan_ledger_files(paths, options).map(PlanLedgerWork::new)
}

fn plan_ledger_records_from_paths(
    paths: &[PathBuf],
    options: &OrgMemorySearchOptions,
) -> Result<PlanLedgerWork, String> {
    if paths.len() < 64 {
        return observe_plan_ledger_files(paths, options);
    }

    // Reading independent files can overlap while the Scheme-AOT graph for
    // each file is built. Bound workers by the work available, not a fixed
    // machine-wide cap, and keep each worker's chunk large enough to amortize
    // thread startup.
    let worker_count = plan_ledger_worker_count(paths.len());
    let chunk_size = paths.len().div_ceil(worker_count);
    let profiled = crate::runtime_profile::is_active();
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in paths.chunks(chunk_size) {
            handles.push(scope.spawn(move || {
                crate::runtime_profile::worker(profiled, || {
                    observe_plan_ledger_files(chunk, options)
                })
            }));
        }

        let mut combined = PlanLedgerWork::new(Vec::new());
        for handle in handles {
            let work = handle
                .join()
                .map_err(|_| "plan ledger worker panicked".to_string())?
                .receive()?;
            combined.records.extend(work.records);
            #[cfg(feature = "runtime-profile")]
            for (stage, nanos) in work.timings {
                *combined.timings.entry(stage).or_default() += nanos;
            }
        }
        Ok(combined)
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

fn collect_plan_ledger_files(
    paths: &[PathBuf],
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    let mut records = Vec::new();
    let config = crate::ParseConfig::default();
    // Bound both file retention and native handoff by the admitted wire limits.
    for chunk in paths.chunks(crate::org_aot::BATCH_MAX_DOCUMENTS) {
        let sources =
            crate::runtime_profile::stage("plan_ledger.source_read", || load_sources(chunk))?;
        let mut remaining = sources.as_slice();
        while !remaining.is_empty() {
            let width = bounded_org_batch_len(remaining);
            if width == 0 {
                let source = &remaining[0];
                let document = parse_org_aot(&source.source).map_err(|error| {
                    format!("{}: Org AOT parse: {error:?}", source.path.display())
                })?;
                if let Some(record) = plan_ledger_record_from_document(
                    &source.path,
                    &source.source,
                    &document,
                    options,
                )? {
                    records.push(record);
                }
                remaining = &remaining[1..];
                continue;
            }
            let (batch, rest) = remaining.split_at(width);
            let inputs = batch
                .iter()
                .map(|source| source.source.as_str())
                .collect::<Vec<_>>();
            let documents = crate::org_aot::parse_org_aot_batch(&inputs, &config)
                .map_err(|error| format!("Org AOT plan ledger batch: {error:?}"))?;
            for (source, document) in batch.iter().zip(documents) {
                let document = document.map_err(|error| {
                    format!("{}: Org AOT parse: {error:?}", source.path.display())
                })?;
                if let Some(record) = plan_ledger_record_from_document(
                    &source.path,
                    &source.source,
                    &document,
                    options,
                )? {
                    records.push(record);
                }
            }
            remaining = rest;
        }
    }
    Ok(records)
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

fn plan_ledger_record_from_document(
    path: &Path,
    source: &str,
    document: &OrgAotDocument,
    options: &OrgMemorySearchOptions,
) -> Result<Option<OrgMemorySearchRecord>, String> {
    crate::runtime_profile::stage("plan_ledger.record_projection", || {
        project_plan_ledger_record(path, source, document, options)
    })
}

fn project_plan_ledger_record(
    path: &Path,
    source: &str,
    document: &OrgAotDocument,
    options: &OrgMemorySearchOptions,
) -> Result<Option<OrgMemorySearchRecord>, String> {
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
