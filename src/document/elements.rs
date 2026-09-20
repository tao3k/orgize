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
    sources.iter().try_fold(Vec::new(), |mut facts, source| {
        facts.extend(index_source(language, &source.path, &source.source)?);
        Ok(facts)
    })
}

pub(super) fn query_project_with_config(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
    _terms: &[String],
    _fields: &[String],
) -> Result<Vec<DocumentElement>, String> {
    let mut files = Vec::new();
    collect_document_paths(language, root, walk_config, &mut files)?;
    files.sort();
    files.dedup();

    index_paths(language, &files)
}

fn index_paths(
    language: DocumentLanguage,
    paths: &[PathBuf],
) -> Result<Vec<DocumentElement>, String> {
    // Small Org files are dominated by worker creation and scheduling; keep
    // that latency out of ordinary project queries and parallelize larger sets.
    const PARALLEL_INDEX_MIN_PATHS: usize = 64;
    if paths.len() < PARALLEL_INDEX_MIN_PATHS {
        return paths.iter().try_fold(Vec::new(), |mut facts, path| {
            facts.extend(index_path(language, path)?);
            Ok(facts)
        });
    }

    let worker_count = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(4)
        .min(paths.len());
    let chunk_size = paths.len().div_ceil(worker_count);
    thread::scope(|scope| {
        paths
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk.iter().try_fold(Vec::new(), |mut facts, path| {
                        facts.extend(index_path(language, path)?);
                        Ok::<_, String>(facts)
                    })
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .try_fold(Vec::new(), |mut facts, task| {
                facts.extend(
                    task.join()
                        .map_err(|_| "document parser worker panicked".to_string())??,
                );
                Ok(facts)
            })
    })
}

fn index_source(
    language: DocumentLanguage,
    path: &Path,
    source: &str,
) -> Result<Vec<DocumentElement>, String> {
    match language {
        DocumentLanguage::Org => Ok(index_org(path, source)),
        DocumentLanguage::Markdown => index_markdown(path, source),
    }
}

/// Filter already-indexed elements with whitespace-delimited text matching.
pub fn filter_elements(elements: &[DocumentElement], query: &str) -> Vec<DocumentElement> {
    elements
        .iter()
        .filter(|element| element.matches(query))
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
    elements
        .into_iter()
        .filter(|element| {
            terms.iter().all(|term| element.matches(term))
                && kinds.iter().all(|kind| element.kind_matches(kind))
                && fields.iter().all(|field| element.field_matches(field))
        })
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

    pub(super) fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return true;
        }
        query.split_whitespace().all(|term| self.matches_term(term))
    }

    fn matches_term(&self, term: &str) -> bool {
        self.kind.to_ascii_lowercase().contains(term)
            || self.source_kind.to_ascii_lowercase().contains(term)
            || self.path.to_ascii_lowercase().contains(term)
            || self.text.to_ascii_lowercase().contains(term)
            || self.content.to_ascii_lowercase().contains(term)
            || self.fields.iter().any(|(key, value)| {
                key.to_ascii_lowercase().contains(term) || value.to_ascii_lowercase().contains(term)
            })
    }

    fn kind_matches(&self, kind: &str) -> bool {
        self.kind.eq_ignore_ascii_case(kind.trim())
    }

    fn field_matches(&self, field: &str) -> bool {
        let field = field.trim();
        if field.is_empty() {
            return true;
        }
        let Some((key, value)) = field.split_once('=') else {
            return self
                .fields
                .iter()
                .any(|(existing_key, _)| existing_key.eq_ignore_ascii_case(field));
        };
        let key = key.trim();
        let value = value.trim();
        if key.eq_ignore_ascii_case("text") {
            return self.text.contains(value);
        }
        self.fields.iter().any(|(existing_key, existing_value)| {
            existing_key.eq_ignore_ascii_case(key) && existing_value.contains(value)
        })
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
