//! Presentation options for Markdown rendered from the Scheme AOT Org graph.

/// Options for graph-backed Markdown export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkdownExportOptions {
    /// Convert Org special strings such as `--`, `---`, and `...` in text.
    pub special_strings: bool,
    /// Expand entity Objects to UTF-8, or preserve their source spelling.
    pub expand_entities: bool,
}

impl Default for MarkdownExportOptions {
    fn default() -> Self {
        Self {
            special_strings: false,
            expand_entities: true,
        }
    }
}
