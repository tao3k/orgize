use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{ElementData, LinkTarget, ObjectData},
    Org,
};

#[test]
fn semantic_ast_projects_radio_links_from_plain_text() {
    let doc = Org::parse("<<<Radio Target>>> links Radio Target, not Radio Targets.").document();

    assert_clean_projection(&doc);
    let objects = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };

    assert!(objects.iter().any(
        |object| matches!(&object.data, ObjectData::RadioTarget(value) if value == "Radio Target")
    ));

    let links = objects
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some((object, link)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(links.len(), 1);
    let (object, link) = links[0];
    assert_eq!(object.ann.raw, "Radio Target");
    assert_eq!(link.path, "Radio Target");
    assert!(matches!(
        &link.target,
        LinkTarget::Internal(target) if target == "Radio Target"
    ));
    assert!(link.has_description);
    assert_eq!(link.raw_description, "Radio Target");
    assert_eq!(link.description.len(), 1);
    assert!(matches!(
        &link.description[0].data,
        ObjectData::Plain(value) if value == "Radio Target"
    ));

    let plain = objects
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Plain(value) => Some(value.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(plain.contains("Radio Targets"));
}
