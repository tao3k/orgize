use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ParseConfig,
    ast::{ElementData, ObjectData},
    config::UseSubSuperscript,
};

#[test]
fn semantic_citation_affixes_respect_parse_config() {
    let config = ParseConfig {
        use_sub_superscript: UseSubSuperscript::Nil,
        ..Default::default()
    };
    let doc = config.parse("See [cite:@doe x_1].").document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let citation = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Citation(citation) => Some(citation),
            _ => None,
        })
        .expect("citation object");

    assert_eq!(citation.references[0].id, "doe");
    assert_eq!(citation.references[0].suffix.len(), 1);
    assert!(matches!(
        &citation.references[0].suffix[0].data,
        ObjectData::Plain(value) if value == "x_1"
    ));
}
