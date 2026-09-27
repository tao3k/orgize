//! Source position lookup for semantic AST annotations.

use rowan::TextSize;

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
        let bytes = source.as_bytes();
        let starts = std::iter::once(0)
            .chain(
                memchr::memchr2_iter(b'\r', b'\n', bytes)
                    .filter(|&index| {
                        bytes[index] != b'\n' || index == 0 || bytes[index - 1] != b'\r'
                    })
                    .map(|index| {
                        if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                            index + 2
                        } else {
                            index + 1
                        }
                    }),
            )
            .collect::<Vec<_>>();
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

#[cfg(test)]
#[path = "../../tests/unit/source_position.rs"]
mod tests;
