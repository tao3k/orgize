use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{ElementData, MarkupKind, ObjectData},
    Org,
};

#[test]
fn semantic_ast_projects_citations() {
    let doc = Org::parse(
        "See [cite/text:global *prefix* ; see /also/ @doe2020 p. *42*; cf. @roe2021; global suffix] and [cite/noauthor/bare:@smith].",
    )
    .document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let citations = paragraph
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Citation(citation) => Some(citation),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(citations.len(), 2);
    assert_eq!(citations[0].style, "text");
    assert_eq!(citations[0].variant, "");
    assert!(matches!(
        &citations[0].prefix[0].data,
        ObjectData::Plain(value) if value == "global "
    ));
    assert!(citations[0].prefix.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert_eq!(citations[0].references[0].id, "doe2020");
    assert!(matches!(
        &citations[0].references[0].prefix[0].data,
        ObjectData::Plain(value) if value == "see "
    ));
    assert!(citations[0].references[0]
        .prefix
        .iter()
        .any(|object| matches!(
            object.data,
            ObjectData::Markup {
                kind: MarkupKind::Italic,
                ..
            }
        )));
    assert!(matches!(
        &citations[0].references[0].suffix[0].data,
        ObjectData::Plain(value) if value == "p. "
    ));
    assert!(citations[0].references[0]
        .suffix
        .iter()
        .any(|object| matches!(
            object.data,
            ObjectData::Markup {
                kind: MarkupKind::Bold,
                ..
            }
        )));
    assert_eq!(citations[0].references[1].id, "roe2021");
    assert!(matches!(
        &citations[0].suffix[0].data,
        ObjectData::Plain(value) if value == "global suffix"
    ));

    assert_eq!(citations[1].style, "noauthor");
    assert_eq!(citations[1].variant, "bare");
    assert_eq!(citations[1].references[0].id, "smith");
}
