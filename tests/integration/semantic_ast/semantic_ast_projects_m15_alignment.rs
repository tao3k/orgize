use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        ElementData, ExportProjectionOptions, FootnoteDefinition, LinkSearchKind, LinkTarget,
        MarkupKind, ObjectData, TargetKind,
    },
};

#[test]
fn semantic_ast_projects_m15_keyword_settings_and_abbreviations() {
    let doc = Org::parse(
        r#"#+TITLE: *Demo* Doc
#+AUTHOR: Alice
#+FILETAGS: :work:rust:
#+FILETAGS: :work:rust:
#+OPTIONS: H:2 -:t e:nil
#+SELECT_TAGS: publish
#+EXCLUDE_TAGS: noexport archived
#+LINK: gh https://github.com/%s
#+LINK: search https://example.test?q=%h
#+ATTR_HTML: :class compact :width "10 em"
[[gh:tao3k/orgize]] [[search:a/b]]
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.filetags, ["work".to_string(), "rust".to_string()]);
    assert_eq!(doc.export_settings.headline_levels, Some(2));
    assert_eq!(doc.export_settings.special_strings, Some(true));
    assert_eq!(doc.export_settings.expand_entities, Some(false));
    assert_eq!(doc.export_settings.select_tags, ["publish".to_string()]);
    assert_eq!(
        doc.export_settings.exclude_tags,
        ["noexport".to_string(), "archived".to_string()]
    );
    assert_eq!(doc.link_abbreviations.len(), 2);
    assert_eq!(doc.link_abbreviations[0].name, "gh");

    let title = doc
        .metadata
        .iter()
        .find(|keyword| keyword.key == "TITLE")
        .expect("title keyword");
    assert!(title.parsed.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));

    let (paragraph, attrs) = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some((objects, &element.affiliated_keywords)),
            _ => None,
        })
        .expect("paragraph");
    let attrs = &attrs[0].attributes;
    assert_eq!(attrs[0].key, "class");
    assert_eq!(attrs[0].value.as_deref(), Some("compact"));
    assert_eq!(attrs[1].key, "width");
    assert_eq!(attrs[1].value.as_deref(), Some("10 em"));

    let links = paragraph
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some(link),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].path(), "gh:tao3k/orgize");
    assert!(matches!(
        &links[0].target,
        LinkTarget::Uri { protocol, path }
            if protocol == "https" && path == "//github.com/tao3k/orgize"
    ));
    assert!(matches!(
        &links[1].target,
        LinkTarget::Uri { protocol, path }
            if protocol == "https" && path == "//example.test?q=a%2Fb"
    ));
}

#[test]
fn semantic_ast_projects_m15_anchors_link_defaults_and_footnotes() {
    let doc = Org::parse(
        r#"* Anchor *Title*
:PROPERTIES:
:CUSTOM_ID: custom-anchor
:ID: org-id-anchor
:END:
[[#custom-anchor]] [[id:org-id-anchor::*Anchor Title]]

Inline [fn::anonymous *inline*] and named [fn:named:explicit inline] then [fn:named].

* Duplicate
* Duplicate
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.sections[0].anchor.as_deref(), Some("custom-anchor"));
    assert_eq!(doc.sections[1].anchor.as_deref(), Some("duplicate"));
    assert_eq!(doc.sections[2].anchor.as_deref(), Some("duplicate-1"));
    assert!(
        doc.targets
            .iter()
            .any(|target| target.kind == TargetKind::Id && target.key == "id:org-id-anchor")
    );

    let links = doc.sections[0]
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .flatten()
        .filter_map(|object| match &object.data {
            ObjectData::Link(link) => Some(link),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(links.len(), 2);
    assert!(links[0].description_or_default().iter().any(
        |object| matches!(object.data, ObjectData::Plain(ref value) if value.contains("Anchor"))
    ));
    assert_eq!(
        links[1].search.as_ref().map(|search| search.kind),
        Some(LinkSearchKind::Headline)
    );

    assert_eq!(doc.footnotes.len(), 2);
    assert!(doc.footnotes.iter().any(|entry| {
        entry.label == "fn-1" && matches!(entry.definition, FootnoteDefinition::Inline(_))
    }));
    assert!(doc.footnotes.iter().any(|entry| entry.label == "named"));
}

#[test]
fn semantic_ast_projects_m15_export_projection() {
    let doc = Org::parse(
        r#"#+FILETAGS: :global:
* Keep :publish:
Text -- more...
* Drop :noexport:
Dropped
* COMMENT Hidden
Hidden
* Archive :ARCHIVE:
Archived
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let projected = doc.project_for_export(&ExportProjectionOptions {
        prune: true,
        special_strings: true,
        headline_level_shift: 1,
        select_tags: vec!["publish".into()],
        exclude_tags: vec!["noexport".into()],
        ..ExportProjectionOptions::default()
    });

    assert_eq!(projected.sections.len(), 1);
    assert_eq!(projected.sections[0].level, 2);
    assert_eq!(projected.sections[0].raw_title, "Keep ");
    let paragraph = match &projected.sections[0].children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    assert!(paragraph
        .iter()
        .any(|object| matches!(object.data, ObjectData::Plain(ref value) if value.contains('\u{2013}') && value.contains('\u{2026}'))));

    let original = match &doc.sections[0].children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    assert!(
        original.iter().any(
            |object| matches!(object.data, ObjectData::Plain(ref value) if value.contains("--"))
        )
    );
}

#[test]
fn semantic_ast_projects_m15_balanced_citation_body() {
    let doc = Org::parse("See [cite:see [nested] @doe p. [42]; cf. @roe].").document();

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
        .expect("citation");
    assert_eq!(citation.references.len(), 2);
    assert_eq!(citation.references[0].id, "doe");
    assert_eq!(citation.references[1].id, "roe");
    assert!(citation.references[0].prefix.iter().any(|object| {
        matches!(object.data, ObjectData::Plain(ref value) if value.contains("[nested]"))
    }));
    assert!(citation.references[0].suffix.iter().any(|object| {
        matches!(object.data, ObjectData::Plain(ref value) if value.contains("[42]"))
    }));
}

#[test]
fn semantic_ast_projects_m15_diagnoses_malformed_citation_segment() {
    let doc = Org::parse("[cite:@ok; @].").document();

    assert!(
        doc.diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.message.contains("malformed citation segment") })
    );
}
