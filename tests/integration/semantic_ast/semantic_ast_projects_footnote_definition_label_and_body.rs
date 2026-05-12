use orgize::{
    ast::{ElementData, ObjectData},
    Org,
};

#[test]
fn semantic_ast_projects_footnote_definition_label_and_body() {
    let doc = Org::parse("#+CAPTION: A note\n[fn:WORD-1] See *bold* text\n").document();

    assert!(doc.diagnostics.is_empty());
    let element = &doc.children[0];
    assert_eq!(element.affiliated_keywords[0].key, "CAPTION");
    assert_eq!(element.affiliated_keywords[0].value, " A note");

    let definition = match &element.data {
        ElementData::FootnoteDef(definition) => definition,
        _ => panic!("expected footnote definition"),
    };
    assert_eq!(definition.label, "WORD-1");

    let body = match &definition.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected footnote body paragraph"),
    };
    assert!(body.iter().any(|object| matches!(
        &object.data,
        ObjectData::Plain(value) if value.contains("*bold*")
    )));
}
