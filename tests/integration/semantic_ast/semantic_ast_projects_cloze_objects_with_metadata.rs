#[cfg(feature = "syntax-org-fc")]
use crate::semantic_ast::support::assert_clean_projection;
#[cfg(feature = "syntax-org-fc")]
use orgize::{
    Org,
    ast::{ElementData, MarkupKind, ObjectData},
};

#[cfg(feature = "syntax-org-fc")]
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

#[cfg(feature = "syntax-org-fc")]
fn cloze_text_uses_graph_children_and_complex_fallback() {
    let source = "{{*大胆*}} and {{*text* [brackets]}}";
    let graph = orgize::org_aot::parse_org_aot(source).expect("Scheme cloze graph");
    assert_eq!(graph.syntax().to_string(), source);
    let records = graph.records();
    let clozes = records
        .iter()
        .filter(|record| record.kind == "cloze")
        .collect::<Vec<_>>();
    assert_eq!(clozes.len(), 2);
    assert_eq!(clozes[0].field("text"), Some("*大胆*"));
    assert!(clozes[0].field("fragment-fallback").is_none());
    assert!(
        clozes[0]
            .child_ids
            .iter()
            .any(|&id| records[id].kind == "bold")
    );
    assert_eq!(
        clozes[1].field("fragment-fallback"),
        Some("*text* [brackets]")
    );

    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };
    let texts = paragraph
        .iter()
        .filter_map(|object| match &object.data {
            ObjectData::Cloze { text, .. } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 2);
    for expected in ["*大胆*", "*text*"] {
        let bold = texts
            .iter()
            .flat_map(|text| text.iter())
            .find(|object| object.ann.raw == expected)
            .expect("bold markup in cloze text");
        assert!(matches!(
            bold.data,
            ObjectData::Markup {
                kind: MarkupKind::Bold,
                ..
            }
        ));
        assert_eq!(
            usize::from(bold.ann.range.start()),
            source.find(expected).unwrap()
        );
    }
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    #[cfg(feature = "syntax-org-fc")]
    (
        "semantic_ast::semantic_ast_projects_cloze_objects_with_metadata::semantic_ast_projects_cloze_objects_with_metadata",
        semantic_ast_projects_cloze_objects_with_metadata,
    ),
    #[cfg(feature = "syntax-org-fc")]
    (
        "semantic_ast::semantic_ast_projects_cloze_objects_with_metadata::cloze_text_uses_graph_children_and_complex_fallback",
        cloze_text_uses_graph_children_and_complex_fallback,
    ),
];
