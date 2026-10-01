//! Source-backed drawer body bounds projected from Scheme graph fields.

use gerbil_parser_rowan::GraphRecord;
use rowan::TextRange;

use super::ParsedAnnotation;

pub(super) fn drawer_body_range(record: &GraphRecord) -> Option<TextRange> {
    let header = record.field_range("header-trivia")?;
    let end = record.field_range("end")?;
    Some(TextRange::new(header.end(), end.start()))
}

pub(super) fn drawer_body(ann: &ParsedAnnotation) -> &str {
    let Some(body_range) = ann.drawer_body_range else {
        return "";
    };
    let start = usize::from(body_range.start() - ann.range.start());
    let end = usize::from(body_range.end() - ann.range.start());
    ann.raw.get(start..end).unwrap_or_default()
}
