//! Document element indexing, filtering, and source selection helpers.

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{
    markdown_elements::index_markdown,
    model::{DocumentElement, DocumentLanguage, DocumentWalkConfig},
    org_elements::index_org,
};

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

    let mut facts = Vec::new();
    for path in files {
        if !path.exists() {
            continue;
        }
        facts.extend(index_path(language, &path)?);
    }
    Ok(facts)
}

/// Index a single document path into parser-owned document elements.
pub fn index_path(language: DocumentLanguage, path: &Path) -> Result<Vec<DocumentElement>, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    index_source(language, path, &source)
}

pub(super) fn query_project_with_config(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
    terms: &[String],
    fields: &[String],
) -> Result<Vec<DocumentElement>, String> {
    let _ = (terms, fields);
    let mut files = Vec::new();
    collect_document_paths(language, root, walk_config, &mut files)?;
    files.sort();
    files.dedup();

    let mut facts = Vec::new();
    for path in files {
        if !path.exists() {
            continue;
        }
        let source =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        facts.extend(index_source(language, &path, &source)?);
    }
    Ok(facts)
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

pub(super) fn count_kind(elements: &[DocumentElement], kind: &str) -> usize {
    elements
        .iter()
        .filter(|element| element.kind == kind)
        .count()
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

fn collect_document_paths(
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

    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let entry_path = entry.path();
        let Some(name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let entry_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", entry_path.display()))?;
        if entry_type.is_dir() {
            if should_skip_project_directory(name, walk_config) {
                continue;
            }
            collect_document_paths(language, &entry_path, walk_config, files)?;
        } else if entry_type.is_file() && language.matches_path(&entry_path) {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn should_skip_project_directory(name: &str, walk_config: &DocumentWalkConfig) -> bool {
    if walk_config
        .include_hidden_dirs
        .iter()
        .any(|included| included == name)
    {
        return false;
    }
    if walk_config
        .ignore_dirs
        .iter()
        .any(|ignored| ignored == name)
    {
        return true;
    }
    name.starts_with('.')
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
