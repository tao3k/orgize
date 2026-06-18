use super::line_index::LineIndex;

#[test]
fn maps_byte_offsets_without_rescanning_source() {
    let source = "alpha\nbeta\n\ngamma";
    let index = LineIndex::new(source);

    assert_eq!(index.line_for(0), 1);
    assert_eq!(index.line_for(5), 1);
    assert_eq!(index.line_for(6), 2);
    assert_eq!(index.line_for(10), 2);
    assert_eq!(index.line_for(11), 3);
    assert_eq!(index.line_for(12), 4);
    assert_eq!(index.line_for(13), 4);
}
