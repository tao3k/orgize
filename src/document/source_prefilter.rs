use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

use super::model::{DocumentLanguage, DocumentWalkConfig};

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
            tokens.extend(terms.iter().filter_map(source_prefilter_term_token));
        }
        tokens.extend(fields.iter().filter_map(source_prefilter_field_token));
        Self { tokens }
    }

    pub(super) fn candidate_paths(
        &self,
        language: DocumentLanguage,
        root: &Path,
        walk_config: &DocumentWalkConfig,
    ) -> Option<Vec<PathBuf>> {
        if self.tokens.is_empty() || !root.is_dir() {
            return None;
        }

        let files = rg_document_files(language, root, walk_config)?;
        let mut candidates: Option<BTreeSet<PathBuf>> = None;
        for token in &self.tokens {
            let mut token_candidates = rg_matching_files(language, root, walk_config, token)?;
            token_candidates.extend(
                files
                    .iter()
                    .filter(|path| path_matches_token(path, token))
                    .cloned(),
            );
            candidates = Some(match candidates {
                Some(existing) => existing
                    .intersection(&token_candidates)
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                None => token_candidates,
            });
        }

        Some(candidates.unwrap_or_default().into_iter().collect())
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

fn rg_document_files(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
) -> Option<Vec<PathBuf>> {
    let mut command = rg_base_command(language, walk_config);
    command.arg("--files").arg(root);
    run_rg_paths(command)
}

fn rg_matching_files(
    language: DocumentLanguage,
    root: &Path,
    walk_config: &DocumentWalkConfig,
    token: &str,
) -> Option<BTreeSet<PathBuf>> {
    let mut command = rg_base_command(language, walk_config);
    command
        .args([
            "--files-with-matches",
            "--fixed-strings",
            "--ignore-case",
            "--",
            token,
        ])
        .arg(root);
    run_rg_paths(command).map(|paths| paths.into_iter().collect())
}

fn rg_base_command(language: DocumentLanguage, walk_config: &DocumentWalkConfig) -> Command {
    let mut command = Command::new("rg");
    command.args(["--color", "never"]);
    if !walk_config.include_hidden_dirs.is_empty() {
        command.arg("--hidden");
        command.args(["--glob", "!.git/**"]);
        command.args(["--glob", "!**/.git/**"]);
    }
    for glob in language_globs(language) {
        command.args(["--glob", glob]);
    }
    for ignored in &walk_config.ignore_dirs {
        command.args(["--glob", &format!("!**/{ignored}/**")]);
    }
    command
}

fn language_globs(language: DocumentLanguage) -> &'static [&'static str] {
    match language {
        DocumentLanguage::Org => &["*.org", "*.org_archive"],
        DocumentLanguage::Markdown => &["*.md", "*.markdown"],
    }
}

fn run_rg_paths(mut command: Command) -> Option<Vec<PathBuf>> {
    let output = command.output().ok()?;
    if !output.status.success() && output.status.code() != Some(1) {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    Some(stdout.lines().map(PathBuf::from).collect())
}

fn path_matches_token(path: &Path, token: &str) -> bool {
    path.display()
        .to_string()
        .to_ascii_lowercase()
        .contains(token)
}

fn source_prefilter_term_token(term: &String) -> Option<String> {
    let term = term.trim();
    if term.is_empty() {
        None
    } else {
        Some(term.to_ascii_lowercase())
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
