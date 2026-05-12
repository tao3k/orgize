use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{ElementData, LinkTarget, MarkupKind, ObjectData},
    Org,
};

#[test]
fn semantic_ast_projects_link_metadata() {
    let image_doc = Org::parse("#+CAPTION: Logo\n[[file:/tmp/logo.svg]]").document();

    assert_clean_projection(&image_doc);
    let image_link = match &image_doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("image link"),
        other => panic!("expected paragraph, got {other:#?}"),
    };
    assert_eq!(image_link.path(), "file:/tmp/logo.svg");
    assert!(matches!(
        &image_link.target,
        LinkTarget::Uri { protocol, path }
            if protocol == "file" && path == "/tmp/logo.svg"
    ));
    assert!(!image_link.has_description());
    assert!(image_link.is_image());
    assert_eq!(image_link.caption.as_ref().unwrap().key, "CAPTION");
    assert_eq!(image_link.caption.as_ref().unwrap().value, " Logo");

    let doc =
        Org::parse("Links [[#heading][*Jump*]] and [[https://example.com][Site]].").document();

    assert_clean_projection(&doc);
    let links = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .filter_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .collect::<Vec<_>>(),
        other => panic!("expected paragraph, got {other:#?}"),
    };

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].path(), "#heading");
    assert!(matches!(
        &links[0].target,
        LinkTarget::Internal(target) if target == "#heading"
    ));
    assert!(links[0].has_description());
    assert_eq!(links[0].raw_description, "*Jump*");
    assert!(links[0].description.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert!(!links[0].is_image());

    assert_eq!(links[1].path(), "https://example.com");
    assert!(matches!(
        &links[1].target,
        LinkTarget::Uri { protocol, path }
            if protocol == "https" && path == "//example.com"
    ));
    assert!(links[1].has_description());
    assert_eq!(links[1].raw_description, "Site");
    assert!(!links[1].is_image());

    let angle_doc = Org::parse("Angle <https://orgmode.org/manual> link.").document();

    assert_clean_projection(&angle_doc);
    let angle_link = match &angle_doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("angle link"),
        other => panic!("expected paragraph, got {other:#?}"),
    };
    assert_eq!(angle_link.path(), "https://orgmode.org/manual");
    assert!(matches!(
        &angle_link.target,
        LinkTarget::Uri { protocol, path }
            if protocol == "https" && path == "//orgmode.org/manual"
    ));
    assert!(!angle_link.has_description());
    assert_eq!(angle_link.raw_description, "");
    assert!(!angle_link.is_image());

    let plain_doc = Org::parse("Plain https://orgmode.org/manual link.").document();

    assert_clean_projection(&plain_doc);
    let plain_link = match &plain_doc.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("plain link"),
        other => panic!("expected paragraph, got {other:#?}"),
    };
    assert_eq!(plain_link.path(), "https://orgmode.org/manual");
    assert!(matches!(
        &plain_link.target,
        LinkTarget::Uri { protocol, path }
            if protocol == "https" && path == "//orgmode.org/manual"
    ));
    assert!(!plain_link.has_description());
    assert_eq!(plain_link.raw_description, "");
    assert!(!plain_link.is_image());
}
