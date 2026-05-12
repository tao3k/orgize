#[cfg(feature = "syntax-org-fc")]
use crate::semantic_ast::support::assert_clean_projection;
#[cfg(feature = "syntax-org-fc")]
use orgize::{
    ast::{ElementData, MarkupKind, ObjectData},
    Org,
};

#[cfg(feature = "syntax-org-fc")]
#[test]
fn semantic_ast_projects_cloze_objects_with_metadata() {
    let doc = Org::parse("{{*text*}{hint}@card-id}").document();

    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        other => panic!("expected paragraph, got {other:#?}"),
    };
    let cloze = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Cloze {
                text,
                raw_text,
                hint,
                id,
                raw,
            } => Some((text, raw_text, hint, id, raw)),
            _ => None,
        })
        .expect("cloze object");

    assert_eq!(cloze.1, "*text*");
    assert_eq!(cloze.2.as_deref(), Some("hint"));
    assert_eq!(cloze.3.as_deref(), Some("card-id"));
    assert_eq!(cloze.4, "{{*text*}{hint}@card-id}");
    assert!(cloze.0.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}
