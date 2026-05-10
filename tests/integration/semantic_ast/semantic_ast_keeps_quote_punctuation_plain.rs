use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{ElementData, MarkupKind, ObjectData},
    Org,
};

#[test]
fn semantic_ast_keeps_quote_punctuation_plain() {
    let doc = Org::parse(
        r#"He said "plain text" and "*bold*" while '/italic/' stays text-adjacent.
"A quoted [[https://example.com][link]]" still contains a link object.
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };

    let plain_text = paragraph
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Plain(value) => Some(value.as_str()),
            _ => None,
        })
        .collect::<String>();

    assert!(plain_text.contains("\"plain text\""));
    assert!(plain_text.contains("while '"));
    assert!(plain_text.contains('"'));
    assert!(paragraph.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert!(paragraph.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Italic,
            ..
        }
    )));
    assert!(paragraph
        .iter()
        .any(|object| matches!(object.data, ObjectData::Link(_))));
}
