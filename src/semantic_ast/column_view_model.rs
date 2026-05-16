//! Column View declarations projected from `COLUMNS` metadata.

use super::{ParsedAnnotation, SourcePosition};

/// One `COLUMNS` declaration visible to document or subtree consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnViewRecord {
    pub source: ColumnViewSource,
    pub scope: ColumnViewScope,
    pub raw: String,
    pub columns: Vec<ColumnViewColumn>,
}

/// Source location for a Column View declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnViewSource {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: u32,
    pub range_end: u32,
}

impl ColumnViewSource {
    pub(crate) fn from_annotation(annotation: &ParsedAnnotation) -> Self {
        Self {
            start: annotation.start,
            end: annotation.end,
            range_start: annotation.range.start().into(),
            range_end: annotation.range.end().into(),
        }
    }
}

/// The Org scope where a `COLUMNS` declaration was found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnViewScope {
    DocumentKeyword,
    DocumentProperty,
    SectionProperty {
        level: usize,
        title: String,
        outline_path: Vec<String>,
    },
}

/// Parsed `%PROPERTY(TITLE){OPERATOR;FORMAT}` column entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnViewColumn {
    pub property: String,
    pub title: Option<String>,
    pub width: Option<usize>,
    pub summary_operator: Option<String>,
    pub summary_format: Option<String>,
    pub raw: String,
}
