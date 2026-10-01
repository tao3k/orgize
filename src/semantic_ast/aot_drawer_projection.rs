//! Source-backed drawer body bounds projected from Scheme graph fields.

use gerbil_parser_rowan::GraphRecord;
use rowan::TextRange;

pub(super) fn drawer_body_range(record: &GraphRecord) -> Option<TextRange> {
    let header = record.field_range("header-trivia")?;
    let end = record.field_range("end")?;
    Some(TextRange::new(header.end(), end.start()))
}
