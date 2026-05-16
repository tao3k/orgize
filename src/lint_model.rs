//! Public lint result model.

use std::path::PathBuf;

use rowan::TextRange;

use crate::ast::SourcePosition;

/// Lint configuration.
///
/// The default keeps linting pure over the provided source string. Set
/// [`include_base_dir`](Self::include_base_dir) when checking `#+INCLUDE:`
/// directives against the filesystem. Set
/// [`attachment_base_dir`](Self::attachment_base_dir) when checking
/// `attachment:` links against the filesystem. Set
/// [`file_base_dir`](Self::file_base_dir) when checking ordinary `file:` links.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintOptions {
    /// Base directory used to resolve relative `#+INCLUDE:` paths.
    pub include_base_dir: Option<PathBuf>,
    /// Base directory used to resolve relative Org attachment directories.
    pub attachment_base_dir: Option<PathBuf>,
    /// Base directory used to resolve relative ordinary `file:` link targets.
    pub file_base_dir: Option<PathBuf>,
}

/// Lint result for one Org source string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintReport {
    pub findings: Vec<LintFinding>,
}

impl LintReport {
    /// Returns true when no lint findings were produced.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// One lint finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintFinding {
    pub code: &'static str,
    pub severity: LintSeverity,
    pub message: String,
    pub location: LintLocation,
}

/// Finding severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintSeverity {
    Error,
    Warning,
}

/// Source location for one finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintLocation {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: usize,
    pub range_end: usize,
}

pub(crate) fn location_for_range(source: &str, range: TextRange) -> LintLocation {
    let start = usize::from(range.start()).min(source.len());
    let end = usize::from(range.end()).min(source.len());
    location_for_offsets(source, start, end)
}

pub(crate) fn location_for_offsets(source: &str, start: usize, end: usize) -> LintLocation {
    let start = start.min(source.len());
    let end = end.min(source.len());
    LintLocation {
        start: position_for_offset(source, start),
        end: position_for_offset(source, end),
        range_start: start,
        range_end: end,
    }
}

fn position_for_offset(source: &str, offset: usize) -> SourcePosition {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..offset].chars().count() + 1;
    SourcePosition { line, column }
}
