use orgize::{
    Org,
    ast::{ElementData, ObjectData},
};

#[test]
fn semantic_ast_keeps_affiliated_keywords_out_of_paragraph_objects() {
    let doc = Org::parse("#+ATTR_HTML: :width 300px\n[[./img/a.jpg]]").document();

    assert!(doc.diagnostics.is_empty());
    let paragraph = &doc.children[0];
    assert_eq!(paragraph.affiliated_keywords.len(), 1);
    assert_eq!(paragraph.affiliated_keywords[0].key, "ATTR_HTML");
    assert_eq!(paragraph.affiliated_keywords[0].value, " :width 300px");

    let objects = match &paragraph.data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };
    assert_eq!(objects.len(), 1);
    assert!(matches!(objects[0].data, ObjectData::Link(_)));
}
