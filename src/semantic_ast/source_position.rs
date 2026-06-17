//! Source position lookup for semantic AST annotations.

use rowan::TextSize;

use crate::syntax::combinator::line_starts_iter;

use super::SourcePosition;

pub(super) struct LineIndex<'a> {
    source: &'a str,
    lines: Vec<LineInfo>,
}

struct LineInfo {
    start: usize,
    char_starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        let starts = line_starts_iter(source).collect::<Vec<_>>();
        let lines = starts
            .iter()
            .enumerate()
            .map(|(index, start)| {
                let end = starts.get(index + 1).copied().unwrap_or(source.len());
                let slice = &source[*start..end];
                let char_starts = if slice.is_ascii() {
                    Vec::new()
                } else {
                    slice
                        .char_indices()
                        .map(|(position, _)| *start + position)
                        .collect()
                };

                LineInfo {
                    start: *start,
                    char_starts,
                }
            })
            .collect();

        Self { source, lines }
    }

    pub(super) fn position(&self, position: TextSize) -> SourcePosition {
        let position = usize::from(position).min(self.source.len());
        let line = match self
            .lines
            .binary_search_by_key(&position, |line| line.start)
        {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_info = &self.lines[line];
        let column = if line_info.char_starts.is_empty() {
            position - line_info.start + 1
        } else {
            line_info
                .char_starts
                .partition_point(|char_start| *char_start < position)
                + 1
        };

        SourcePosition {
            line: line + 1,
            column,
        }
    }
}
