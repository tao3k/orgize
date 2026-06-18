//! Shared document provider data types for commands, indexing, and packet rendering.

use std::path::Path;

/// Document syntax family handled by the embedded document provider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentLanguage {
    /// Org syntax parsed by orgize.
    Org,
    /// Markdown syntax parsed by comrak.
    Markdown,
}

/// Parser-emitted document element used by search and query output.
#[derive(Clone, Debug)]
pub struct DocumentElement {
    /// Agent-facing semantic kind, such as `heading`, `task`, or `checklistItem`.
    pub kind: &'static str,
    /// Parser-specific source node kind.
    pub source_kind: &'static str,
    /// Display path for the source document.
    pub path: String,
    /// One-based start line.
    pub line: usize,
    /// One-based inclusive end line.
    pub end_line: usize,
    /// Provider-owned key/value facts for the element.
    pub fields: Vec<(String, String)>,
    /// Compact display text for seed and metadata views.
    pub text: String,
    /// Source-backed content used by content views and term matching.
    pub content: String,
}

/// Directory walk policy for document project indexing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentWalkConfig {
    /// Directory names skipped during project walks.
    pub ignore_dirs: Vec<String>,
    /// Hidden directory names that should still be walked.
    pub include_hidden_dirs: Vec<String>,
}

impl Default for DocumentWalkConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: default_ignore_dirs()
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            include_hidden_dirs: Vec::new(),
        }
    }
}

impl DocumentWalkConfig {
    /// Creates a document walk policy from caller-owned directory lists.
    pub fn new(ignore_dirs: Vec<String>, include_hidden_dirs: Vec<String>) -> Self {
        Self {
            ignore_dirs,
            include_hidden_dirs,
        }
    }
}

impl DocumentLanguage {
    /// Stable language id used by CLI and packet output.
    pub fn id(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::Markdown => "md",
        }
    }

    /// Public command prefix for the language document provider.
    pub fn command_prefix(self) -> &'static str {
        match self {
            Self::Org => "asp org",
            Self::Markdown => "asp md",
        }
    }

    /// Parser authority that owns element extraction for this syntax.
    pub fn parser_authority(self) -> &'static str {
        match self {
            Self::Org => "orgize",
            Self::Markdown => "comrak",
        }
    }

    pub(super) fn matches_path(self, path: &Path) -> bool {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return false;
        };
        match self {
            Self::Org => matches!(extension, "org" | "org_archive"),
            Self::Markdown => matches!(extension, "md" | "markdown"),
        }
    }
}

fn default_ignore_dirs() -> &'static [&'static str] {
    &[
        "target",
        "node_modules",
        "dist",
        "build",
        "__pycache__",
        "venv",
        "vendor",
    ]
}
