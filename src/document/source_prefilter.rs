use std::path::Path;

pub(super) struct SourcePrefilter {
    tokens: Vec<String>,
}

impl SourcePrefilter {
    pub(super) fn new(terms: &[String], fields: &[String]) -> Self {
        let mut tokens = Vec::new();
        if !terms
            .iter()
            .flat_map(|term| term.split_whitespace())
            .any(is_document_metadata_token)
        {
            tokens.extend(
                terms
                    .iter()
                    .flat_map(|term| term.split_whitespace())
                    .map(|term| term.to_ascii_lowercase()),
            );
        }
        tokens.extend(fields.iter().filter_map(source_prefilter_field_token));
        Self { tokens }
    }

    pub(super) fn matches_path_or_source(&self, path: &Path, source: &str) -> bool {
        if self.tokens.is_empty() {
            return true;
        }
        let path = path.display().to_string().to_ascii_lowercase();
        let source = source.to_ascii_lowercase();
        self.tokens
            .iter()
            .all(|token| path.contains(token) || source.contains(token))
    }
}

fn source_prefilter_field_token(field: &String) -> Option<String> {
    let (_, value) = field.split_once('=')?;
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_ascii_lowercase())
    }
}

fn is_document_metadata_token(term: &str) -> bool {
    matches!(
        term.to_ascii_lowercase().as_str(),
        "heading"
            | "task"
            | "property"
            | "planning"
            | "table"
            | "paragraph"
            | "block"
            | "list"
            | "listitem"
            | "checklistitem"
            | "link"
            | "image"
            | "headline"
            | "propertydrawer"
            | "syntaxplanning"
            | "orgtable"
            | "sourceblock"
            | "exportblock"
            | "syntaxlist"
            | "syntaxlistitem"
            | "syntaxlink"
    )
}
