use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ElementData, FileLinkPathKind, LinkSearchKind, LinkTarget, MarkupKind, ObjectData},
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
    let file = image_link.file.as_ref().expect("file metadata");
    assert_eq!(file.protocol, "file");
    assert_eq!(file.path, "/tmp/logo.svg");
    assert_eq!(file.path_kind, FileLinkPathKind::Absolute);
    assert_eq!(file.search, None);
    assert_eq!(image_link.caption.as_ref().unwrap().key, "CAPTION");
    assert_eq!(image_link.caption.as_ref().unwrap().value, " Logo");

    let searched_file = Org::parse("[[file:notes/demo.org::*Target Heading][target]]").document();
    let searched_link = match &searched_file.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("searched file link"),
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let file = searched_link.file.as_ref().expect("file search metadata");
    assert_eq!(file.path, "notes/demo.org");
    assert_eq!(file.path_kind, FileLinkPathKind::Relative);
    assert!(file.search.as_ref().is_some_and(|search| {
        search.raw == "*Target Heading"
            && search.kind == LinkSearchKind::Headline
            && search.normalized == "target heading"
    }));

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
