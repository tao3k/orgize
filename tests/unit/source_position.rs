use super::LineIndex;
use crate::ast::SourcePosition;

#[test]
fn mixed_newlines_and_unicode_keep_source_positions() {
    let lines = LineIndex::new("a\r\nβ\nc\rd");
    for (offset, expected) in [
        (0, SourcePosition { line: 1, column: 1 }),
        (3, SourcePosition { line: 2, column: 1 }),
        (5, SourcePosition { line: 2, column: 2 }),
        (6, SourcePosition { line: 3, column: 1 }),
        (8, SourcePosition { line: 4, column: 1 }),
    ] {
        assert_eq!(lines.position(offset.into()), expected);
    }
}
