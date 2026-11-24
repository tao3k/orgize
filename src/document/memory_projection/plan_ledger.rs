//! Bounded host I/O and handoff; all Org syntax comes from the native parser.
use super::records::{
    file_matches_options, memory_search_record_matches_options, memory_search_root,
};
use super::{OrgMemorySearchOptions, OrgMemorySearchRecord};
use crate::document::elements::{DocumentSource, bounded_org_batch_len, load_sources};
use crate::org_aot::{
    PlanLedgerHeadline, parse_org_aot, parse_plan_ledger_batch, plan_ledger_headline,
};
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
    time::Instant,
};

pub(super) fn query_plan_ledger_records(
    root: &Path,
    walk_config: &DocumentWalkConfig,
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    let started = Instant::now();
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
    if std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER").is_some()
        || std::env::var_os("ORGIZE_PROFILE_PLAN_LEDGER_STAGES").is_some()
    {
        eprintln!(
            "plan ledger stages: files={} reader_threads={} walk={walk_elapsed:?} projection={projection_elapsed:?} total={:?}",
            files.len(),
            usize::from(files.len() >= crate::org_aot::BATCH_MAX_DOCUMENTS),
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
    let chunks = paths.chunks(crate::org_aot::BATCH_MAX_DOCUMENTS);
    if paths.len() < crate::org_aot::BATCH_MAX_DOCUMENTS {
        return collect_plan_ledger_batches(chunks.map(load_plan_ledger_sources), options);
    }

    // One bounded reader overlaps file I/O with the existing Scheme owner.
    // Rendezvous retains at most the current and next source batch; waiting
    // native requests cannot multiply file readers or their retained inputs.
    let profiled = crate::runtime_profile::is_active();
    thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(0);
        let reader = scope.spawn(move || {
            crate::runtime_profile::worker(profiled, || {
                for chunk in chunks {
                    let sources = load_plan_ledger_sources(chunk);
                    let failed = sources.is_err();
                    if sender.send(sources).is_err() || failed {
                        break;
                    }
                }
            })
        });
        let result = collect_plan_ledger_batches(receiver.iter(), options);
        // A parser error must disconnect a reader blocked on its next handoff
        // before the scope joins it.
        drop(receiver);
        reader
            .join()
            .map_err(|_| "plan ledger source reader panicked".to_string())?
            .receive();
        result
    })
}

fn load_plan_ledger_sources(paths: &[PathBuf]) -> Result<Vec<DocumentSource>, String> {
    crate::runtime_profile::stage("plan_ledger.source_read", || load_sources(paths))
}

fn collect_plan_ledger_batches(
    batches: impl IntoIterator<Item = Result<Vec<DocumentSource>, String>>,
    options: &OrgMemorySearchOptions,
) -> Result<Vec<OrgMemorySearchRecord>, String> {
    let mut records = Vec::new();
    let config = crate::ParseConfig::default();
    for sources in batches {
        let sources = sources?;
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
                    || plan_ledger_headline(&document),
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
            let documents = parse_plan_ledger_batch(&inputs, &config)
                .map_err(|error| format!("Org AOT plan ledger batch: {error:?}"))?;
            for (source, document) in batch.iter().zip(documents) {
                let document = document.map_err(|error| {
                    format!("{}: Org AOT parse: {error:?}", source.path.display())
                })?;
                if let Some(record) = plan_ledger_record_from_document(
                    &source.path,
                    &source.source,
                    || document.headline(),
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

fn plan_ledger_record_from_document<'a>(
    path: &Path,
    source: &str,
    headline: impl FnOnce() -> Option<PlanLedgerHeadline<'a>>,
    options: &OrgMemorySearchOptions,
) -> Result<Option<OrgMemorySearchRecord>, String> {
    crate::runtime_profile::stage("plan_ledger.record_projection", || {
        project_plan_ledger_record(path, source, headline(), options)
    })
}

fn project_plan_ledger_record(
    path: &Path,
    source: &str,
    headline: Option<PlanLedgerHeadline<'_>>,
    options: &OrgMemorySearchOptions,
) -> Result<Option<OrgMemorySearchRecord>, String> {
    let Some(PlanLedgerHeadline {
        records,
        headline,
        todo,
        title,
        closed,
    }) = headline
    else {
        return Ok(None);
    };
    let title = title.ok_or_else(|| format!("{}: headline lacks an AOT title", path.display()))?;
    let tags = headline.values("tag").map(str::to_owned).collect();
    let state = if closed {
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
    let end_line = LineIndex::line_for_single_offset(
        source,
        usize::from(headline.range.end()).saturating_sub(1),
    );
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

#[cfg(test)]
#[path = "../../../tests/unit/plan_ledger_source.rs"]
mod tests;
