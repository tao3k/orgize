//! Shared helpers for projecting lossless syntax into semantic AST nodes.

use rowan::{TextRange, TextSize};

use crate::syntax::{SyntaxElement, SyntaxNode};

use super::block_metadata::split_block_lines;

pub(super) fn parse_token<T>(value: &str) -> Option<T>
where
    T: std::str::FromStr,
{
    value.parse().ok()
}

pub(super) fn range_from_elements(elements: &[SyntaxElement]) -> Option<TextRange> {
    let start = elements.first()?.text_range().start();
    let end = elements.last()?.text_range().end();
    Some(TextRange::new(start, end))
}

pub(super) fn strip_pair(value: &str) -> &str {
    value
        .char_indices()
        .nth(1)
        .map(|(start, _)| {
            let end = value
                .char_indices()
                .last()
                .map(|(index, _)| index)
                .unwrap_or(value.len());
            &value[start..end]
        })
        .unwrap_or("")
}

pub(super) fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::new(start as u32), TextSize::new(end as u32))
}

pub(super) fn block_content_line_ranges(content: &SyntaxNode, source: &str) -> Vec<TextRange> {
    source_line_ranges(usize::from(content.text_range().start()), source)
}

pub(super) fn source_line_ranges(base: usize, source: &str) -> Vec<TextRange> {
    split_block_lines(source)
        .into_iter()
        .map(|line| text_range(base + line.start, base + line.end))
        .collect()
}

pub(super) fn position_range(range: TextRange, base: usize) -> TextRange {
    text_range(
        base + usize::from(range.start()),
        base + usize::from(range.end()),
    )
}

pub(super) fn trimmed_range(value: &str) -> Option<(usize, usize)> {
    let start = value
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))?;
    let end = value
        .char_indices()
        .rfind(|(_, ch)| !ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())?;
    Some((start, end))
}

pub(super) fn strip_wrapping(value: &str, prefix: &str, suffix: &str) -> String {
    value
        .strip_prefix(prefix)
        .and_then(|inner| inner.strip_suffix(suffix))
        .unwrap_or(value)
        .to_string()
}
