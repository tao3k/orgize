//! Document element indexing, filtering, and source selection helpers.

use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

use super::{
    markdown_elements::index_markdown,
    model::{DocumentElement, DocumentLanguage, DocumentWalkConfig},
    org_elements::index_org,
    query_match::PreparedQuery,
};

pub(super) struct DocumentSource {
    pub(super) path: PathBuf,
    pub(super) source: String,
}

/// Index all document files under `root` with the default walk policy.
pub fn index_project(
    language: DocumentLanguage,
    root: &Path,
) -> Result<Vec<DocumentElement>, String> {
    index_project_with_config(language, root, &DocumentWalkConfig::default())
}

/// Index all document files under `root` with an explicit walk policy.
pub fn index_project_with_config(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
) -> Result<Vec<DocumentElement>, String> {
    let mut files = Vec::new();
    collect_document_paths(language, root, walk_config, &mut files)?;
    files.sort();
    files.dedup();

    index_paths(language, &files)
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

/// Index a single document path into parser-owned document elements.
pub fn index_path(language: DocumentLanguage, path: &Path) -> Result<Vec<DocumentElement>, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    index_source(language, path, &source)
}

pub(super) fn load_sources(paths: &[PathBuf]) -> Result<Vec<DocumentSource>, String> {
    paths
        .iter()
        .map(|path| {
            let source =
                fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            Ok(DocumentSource {
                path: path.clone(),
                source,
            })
        })
        .collect()
}

pub(super) fn index_sources(
    language: DocumentLanguage,
    sources: &[DocumentSource],
) -> Result<Vec<DocumentElement>, String> {
    index_sources_with_query(language, sources, None)
}

fn index_sources_with_query(
    language: DocumentLanguage,
    sources: &[DocumentSource],
    query: Option<&PreparedQuery>,
) -> Result<Vec<DocumentElement>, String> {
    if language == DocumentLanguage::Org {
        let config = crate::ParseConfig::default();
        let mut remaining = sources;
        let mut facts = Vec::new();
        while !remaining.is_empty() {
            let width = bounded_org_batch_len(remaining);
            if width == 0 {
                // An oversized document uses the same native single-document API.
                let elements = index_source(language, &remaining[0].path, &remaining[0].source)?;
                facts.extend(
                    elements
                        .into_iter()
                        .filter(|element| query.is_none_or(|query| query.matches(element))),
                );
                crate::runtime_profile::query_batch_completed(1);
                remaining = &remaining[1..];
                continue;
            }
            let (batch, rest) = remaining.split_at(width);
            let inputs = batch
                .iter()
                .map(|source| source.source.as_str())
                .collect::<Vec<_>>();
            let documents = crate::org_aot::parse_org_aot_batch(&inputs, &config)
                .map_err(|error| format!("Org AOT batch: {error:?}"))?;
            for (source, document) in batch.iter().zip(documents) {
                let document =
                    document.map_err(|error| format!("{}: {error:?}", source.path.display()))?;
                facts.extend(crate::runtime_profile::stage(
                    "query.fact_projection",
                    || {
                        super::org_elements::index_org_document_with_query(
                            &source.path,
                            &source.source,
                            &document,
                            query,
                        )
                    },
                )?);
            }
            remaining = rest;
            crate::runtime_profile::query_batch_completed(width);
        }
        return Ok(facts);
    }
    sources.iter().try_fold(Vec::new(), |mut facts, source| {
        facts.extend(
            index_source(language, &source.path, &source.source)?
                .into_iter()
                .filter(|element| query.is_none_or(|query| query.matches(element))),
        );
        Ok(facts)
    })
}

pub(super) fn bounded_org_batch_len(sources: &[DocumentSource]) -> usize {
    let mut bytes = 0_usize;
    let mut count = 0;
    for source in sources.iter().take(crate::org_aot::BATCH_MAX_DOCUMENTS) {
        let Some(size) = bytes.checked_add(source.source.len()) else {
            break;
        };
        if size > crate::org_aot::BATCH_MAX_SOURCE_BYTES {
            break;
        }
        bytes = size;
        count += 1;
    }
    count
}

pub(super) fn query_project_with_config(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
    terms: &[String],
    fields: &[String],
) -> Result<Vec<DocumentElement>, String> {
    let mut files = Vec::new();
    collect_document_paths(language, root, walk_config, &mut files)?;
    files.sort();
    files.dedup();

    let query = PreparedQuery::new(terms, &[], fields);
    index_paths_with_query(language, &files, Some(&query))
}

fn index_paths(
    language: DocumentLanguage,
    paths: &[PathBuf],
) -> Result<Vec<DocumentElement>, String> {
    index_paths_with_query(language, paths, None)
}

fn index_paths_with_query(
    language: DocumentLanguage,
    paths: &[PathBuf],
    query: Option<&PreparedQuery>,
) -> Result<Vec<DocumentElement>, String> {
    let sources = crate::runtime_profile::stage("query.source_load", || load_sources(paths))?;
    let total_bytes = sources
        .iter()
        .map(|source| source.source.len() as u64)
        .sum();
    if should_index_sequentially(language, sources.len(), total_bytes) {
        return index_sources_with_query(language, &sources, query);
    }

    let worker_count = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(4)
        .min(sources.len());
    let chunk_size = sources.len().div_ceil(worker_count);
    let observe_workers = crate::runtime_profile::is_active();
    thread::scope(|scope| {
        sources
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    crate::runtime_profile::worker(observe_workers, || {
                        index_sources_with_query(language, chunk, query)
                    })
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .try_fold(Vec::new(), |mut facts, task| {
                facts.extend(
                    task.join()
                        .map_err(|_| "document parser worker panicked".to_string())?
                        .receive()?,
                );
                Ok(facts)
            })
    })
}

pub(super) fn should_index_sequentially(
    language: DocumentLanguage,
    path_count: usize,
    total_bytes: u64,
) -> bool {
    const PARALLEL_INDEX_MIN_PATHS: usize = 16;
    const SMALL_ORG_BATCH_MAX_PATHS: usize = 63;
    const SMALL_ORG_BATCH_MAX_BYTES: u64 = 64 * 1024;

    if path_count < PARALLEL_INDEX_MIN_PATHS {
        return true;
    }
    if language != DocumentLanguage::Org || path_count > SMALL_ORG_BATCH_MAX_PATHS {
        return false;
    }
    total_bytes <= SMALL_ORG_BATCH_MAX_BYTES
}

fn index_source(
    language: DocumentLanguage,
    path: &Path,
    source: &str,
) -> Result<Vec<DocumentElement>, String> {
    match language {
        DocumentLanguage::Org => index_org(path, source),
        DocumentLanguage::Markdown => index_markdown(path, source),
    }
}

/// Filter already-indexed elements with whitespace-delimited text matching.
pub fn filter_elements(elements: &[DocumentElement], query: &str) -> Vec<DocumentElement> {
    let query = PreparedQuery::new(&[query.to_owned()], &[], &[]);
    elements
        .iter()
        .filter(|element| query.matches(element))
        .cloned()
        .collect()
}

/// Filter already-indexed elements with structured term, kind, and field predicates.
pub fn filter_elements_by_query(
    elements: Vec<DocumentElement>,
    terms: &[String],
    kinds: &[String],
    fields: &[String],
) -> Vec<DocumentElement> {
    let query = PreparedQuery::new(terms, kinds, fields);
    elements
        .into_iter()
        .filter(|element| query.matches(element))
        .collect()
}

pub(super) fn last_existing_path(args: &[String]) -> Option<PathBuf> {
    args.iter()
        .rev()
        .filter(|arg| !arg.starts_with('-'))
        .map(PathBuf::from)
        .find(|path| path.exists())
}

pub(super) fn option_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then_some(window[1].as_str()))
}

pub(super) fn option_values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter_map(|window| (window[0] == name).then_some(window[1].clone()))
        .collect()
}

pub(super) fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

pub(super) fn display_path(path: &Path) -> String {
    path.display().to_string()
}

pub(super) fn escape_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

pub(super) fn collect_document_paths(
    language: DocumentLanguage,
    path: &Path,
    walk_config: &DocumentWalkConfig,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if metadata.is_file() {
        if language.matches_path(path) {
            files.push(path.to_path_buf());
            return Ok(());
        }
        return Err(format!(
            "{}: expected {} file",
            path.display(),
            language.id()
        ));
    }
    if !metadata.is_dir() {
        return Err(format!("{}: unsupported path type", path.display()));
    }

    let ignored_directories = walk_config.ignore_dirs.clone();
    let included_hidden_directories = walk_config.include_hidden_dirs.clone();
    let mut builder = ignore::WalkBuilder::new(path);
    builder
        .standard_filters(true)
        .hidden(false)
        .follow_links(false)
        .filter_entry(move |entry| {
            if entry.depth() == 0 || !entry.file_type().is_some_and(|kind| kind.is_dir()) {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            if ignored_directories
                .iter()
                .any(|ignored| ignored == name.as_ref())
            {
                return false;
            }
            !name.starts_with('.')
                || included_hidden_directories
                    .iter()
                    .any(|included| included == name.as_ref())
        });
    for entry in builder.build() {
        let entry = entry.map_err(|error| format!("{}: {error}", path.display()))?;
        if entry.depth() == 0 {
            continue;
        }
        if entry.file_type().is_some_and(|kind| kind.is_file())
            && language.matches_path(entry.path())
        {
            files.push(entry.into_path());
        }
    }
    files.sort();
    Ok(())
}

impl DocumentElement {
    pub(super) fn render(&self) -> String {
        let mut output = format!(
            "|{} {}:{}-{}",
            self.kind, self.path, self.line, self.end_line
        );
        output.push_str(" sourceKind=\"");
        output.push_str(self.source_kind);
        output.push('"');
        for (key, value) in &self.fields {
            output.push(' ');
            output.push_str(key);
            output.push_str("=\"");
            output.push_str(&escape_field(value));
            output.push('"');
        }
        if !self.text.is_empty() {
            output.push_str(" text=\"");
            output.push_str(&escape_field(&self.text));
            output.push('"');
        }
        output
    }

    pub(super) fn content_text(&self) -> String {
        if !self.content.trim().is_empty() {
            return self.content.clone();
        }
        if !self.text.trim().is_empty() {
            return self.text.clone();
        }
        self.fields
            .iter()
            .find(|(key, value)| {
                matches!(
                    key.as_str(),
                    "title" | "value" | "description" | "target" | "lang"
                ) && !value.trim().is_empty()
            })
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    }
}
