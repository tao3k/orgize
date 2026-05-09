use super::*;

#[test]
fn semantic_ast_projects_inline_babel_and_footnote_details() {
    let doc = Org::parse(
        r#"call_square[:results output](4)[:results html] and src_rust[:exports code]{let x = 1;} and [fn:note:See *bold* text]."#,
    )
    .document();

    assert!(doc.diagnostics.is_empty());
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };

    let inline_call = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                ..
            } => Some((name, arguments, header, end_header)),
            _ => None,
        })
        .expect("inline call object");
    assert_eq!(inline_call.0, "square");
    assert_eq!(inline_call.1, "4");
    assert_eq!(inline_call.2.as_deref(), Some(":results output"));
    assert_eq!(inline_call.3.as_deref(), Some(":results html"));

    let inline_src = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                ..
            } => Some((language, parameters, value)),
            _ => None,
        })
        .expect("inline src object");
    assert_eq!(inline_src.0, "rust");
    assert_eq!(inline_src.1.as_deref(), Some(":exports code"));
    assert_eq!(inline_src.2, "let x = 1;");

    let footnote = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::FootnoteRef { label, definition } => Some((label, definition)),
            _ => None,
        })
        .expect("footnote ref object");
    assert_eq!(footnote.0.as_deref(), Some("note"));
    assert!(footnote.1.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}
