use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{ElementData, MarkupKind, ObjectData},
    Org,
};

#[test]
fn semantic_ast_projects_object_gap_repairs() {
    let doc = Org::parse(
        r#"[[https://example.com][*bold* description]] [2003-09-16 Tue 09:39]--[2003-09-16 Tue 10:39] {{{macro(1\,a, two)}}}"#,
    )
    .document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };

    let link_description = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Link(link) => Some(&link.description),
            _ => None,
        })
        .expect("link object");
    assert!(link_description.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));

    let timestamp = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Timestamp(timestamp) => Some(timestamp),
            _ => None,
        })
        .expect("timestamp object");
    assert!(timestamp.is_range);
    assert_eq!(timestamp.start.as_ref().unwrap().hour, Some(9));
    assert_eq!(timestamp.start.as_ref().unwrap().minute, Some(39));
    assert_eq!(timestamp.end.as_ref().unwrap().hour, Some(10));
    assert_eq!(timestamp.end.as_ref().unwrap().minute, Some(39));

    let macro_arguments = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Macro { name, arguments } if name == "macro" => Some(arguments),
            _ => None,
        })
        .expect("macro object");
    assert_eq!(macro_arguments, &["1,a".to_string(), "two".to_string()]);
}
