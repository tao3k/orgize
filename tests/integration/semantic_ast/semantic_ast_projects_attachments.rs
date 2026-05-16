use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        AttachmentDirectorySource, AttachmentIdPathLayout, AttachmentLinkSearchKind, ElementData,
        ObjectData,
    },
    Org,
};

const SOURCE: &str = r#"* Project :ATTACH:
:PROPERTIES:
:DIR: assets
:END:
See [[attachment:diagram.png::255]].
** Child
See [[attachment:child.txt::*Heading]].
* ID backed
:PROPERTIES:
:ID: 95d50008-c12e-479f-a4f2-cc0238205319
:END:
See [[attachment:info.org::#custom]].
* Legacy
:PROPERTIES:
:ATTACH_DIR: legacy
:END:
See [[attachment:old.pdf::/needle/]].
"#;

#[test]
fn semantic_ast_projects_attachment_directories_and_links() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let project = &doc.sections[0];
    assert!(project.attachment.has_attach_tag);
    let project_dir = project
        .attachment
        .directory
        .as_ref()
        .expect("project attachment directory");
    assert_eq!(project_dir.path, "assets");
    assert_eq!(project_dir.source, AttachmentDirectorySource::DirProperty);

    let project_link = first_section_link(project);
    let project_attachment = project_link.attachment.as_ref().expect("attachment link");
    assert_eq!(project_attachment.path, "diagram.png");
    assert_eq!(
        project_attachment
            .search
            .as_ref()
            .map(|search| (&search.raw, search.kind)),
        Some((&"255".to_string(), AttachmentLinkSearchKind::LineNumber))
    );

    let child = &project.subsections[0];
    assert_eq!(
        child
            .attachment
            .directory
            .as_ref()
            .map(|directory| directory.path.as_str()),
        Some("assets")
    );
    let child_attachment = first_section_link(child)
        .attachment
        .as_ref()
        .expect("child attachment link");
    assert_eq!(
        child_attachment
            .search
            .as_ref()
            .map(|search| (&search.raw, search.kind)),
        Some((&"*Heading".to_string(), AttachmentLinkSearchKind::Headline))
    );

    let id_backed = &doc.sections[1];
    let id_dir = id_backed
        .attachment
        .directory
        .as_ref()
        .expect("id attachment directory");
    assert_eq!(id_dir.path, "data/95/d50008-c12e-479f-a4f2-cc0238205319");
    assert!(matches!(
        &id_dir.source,
        AttachmentDirectorySource::IdDerived {
            id,
            layout: AttachmentIdPathLayout::Uuid,
        } if id == "95d50008-c12e-479f-a4f2-cc0238205319"
    ));

    let legacy = &doc.sections[2];
    assert_eq!(
        legacy
            .attachment
            .directory
            .as_ref()
            .map(|directory| (&directory.source, directory.path.as_str())),
        Some((
            &AttachmentDirectorySource::LegacyAttachDirProperty,
            "legacy"
        ))
    );
    assert_eq!(
        first_section_link(legacy)
            .attachment
            .as_ref()
            .and_then(|attachment| attachment.search.as_ref())
            .map(|search| search.kind),
        Some(AttachmentLinkSearchKind::Regexp)
    );

    insta::assert_debug_snapshot!(
        "semantic_ast__semantic_attachment_projection",
        doc.section_index_records()
    );
}

fn first_section_link(
    section: &orgize::ast::Section<orgize::ast::ParsedAnnotation>,
) -> &orgize::ast::Link<orgize::ast::ParsedAnnotation> {
    match &section.children[0].data {
        ElementData::Paragraph(objects) => objects
            .iter()
            .find_map(|object| match &object.data {
                ObjectData::Link(link) => Some(link),
                _ => None,
            })
            .expect("attachment link"),
        other => panic!("expected paragraph, got {other:#?}"),
    }
}
