use super::*;

#[test]
fn semantic_annotations_handle_parser_line_endings_and_utf8_columns() {
    let doc = Org::parse("* A\réé *bold*").document();

    assert_clean_projection(&doc);
    let paragraph = &doc.sections[0].children[0];
    assert_eq!(paragraph.ann.start, SourcePosition { line: 2, column: 1 });

    let objects = match &paragraph.data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let bold = objects
        .iter()
        .find(|object| {
            matches!(
                object.data,
                ObjectData::Markup {
                    kind: MarkupKind::Bold,
                    ..
                }
            )
        })
        .expect("bold object");

    assert_eq!(bold.ann.start, SourcePosition { line: 2, column: 4 });
    assert_eq!(
        bold.ann.end,
        SourcePosition {
            line: 2,
            column: 10
        }
    );
}
