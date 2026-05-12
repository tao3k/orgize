use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AstRef, ElementData, LinkTarget, ObjectData, TargetKind},
    Org,
};

#[test]
fn semantic_ast_resolves_document_local_internal_links() {
    let doc = Org::parse(
        r#"* Anchor Heading
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
See [[*Anchor Heading][headline]], [[#heading-id][custom id]], [[target-one][target]], [[fn:note][footnote]], and [[coderef:init][code]].

Paragraph with <<target-one>> and <<<radio-one>>>.

#+begin_src rust -l "ref:%s"
let x = 1; ref:init
#+end_src

[fn:note] Footnote body
"#,
    )
    .document();

    assert_clean_projection(&doc);

    assert!(doc
        .targets
        .iter()
        .any(|target| target.key == "Anchor Heading" && target.kind == TargetKind::Headline));
    assert!(doc
        .targets
        .iter()
        .any(|target| target.key == "#heading-id" && target.kind == TargetKind::CustomId));
    assert!(doc
        .targets
        .iter()
        .any(|target| target.key == "target-one" && target.kind == TargetKind::Target));
    assert!(doc
        .targets
        .iter()
        .any(|target| target.key == "radio-one" && target.kind == TargetKind::RadioTarget));
    assert!(doc
        .targets
        .iter()
        .any(|target| target.key == "fn:note" && target.kind == TargetKind::FootnoteDefinition));
    assert!(doc
        .targets
        .iter()
        .any(|target| target.key == "coderef:init" && target.kind == TargetKind::CodeRef));

    let mut link_targets = Vec::new();
    doc.visit(|node| {
        if let AstRef::Object(object) = node {
            if let ObjectData::Link(link) = &object.data {
                link_targets.push((link.path().to_string(), link.target.clone()));
            }
        }
    });

    assert!(link_targets.iter().any(|(path, target)| matches!(
        (path.as_str(), target),
        ("*Anchor Heading", LinkTarget::Internal(value)) if value == "Anchor Heading"
    )));
    assert!(link_targets.iter().any(|(path, target)| matches!(
        (path.as_str(), target),
        ("#heading-id", LinkTarget::Internal(value)) if value == "#heading-id"
    )));
    assert!(link_targets.iter().any(|(path, target)| matches!(
        (path.as_str(), target),
        ("target-one", LinkTarget::Internal(value)) if value == "target-one"
    )));
    assert!(link_targets.iter().any(|(path, target)| matches!(
        (path.as_str(), target),
        ("fn:note", LinkTarget::Internal(value)) if value == "fn:note"
    )));
    assert!(link_targets.iter().any(|(path, target)| matches!(
        (path.as_str(), target),
        ("coderef:init", LinkTarget::Internal(value)) if value == "coderef:init"
    )));

    let target_count = doc.fold(0usize, |count, node| match node {
        AstRef::TargetDefinition(_) => count + 1,
        _ => count,
    });
    assert_eq!(target_count, doc.targets.len());
}

#[test]
fn semantic_ast_diagnoses_ambiguous_and_missing_strict_internal_links() {
    let doc = Org::parse(
        r#"<<same>> <<same>>
[[same]] [[fn:missing]] [[coderef:missing]] [[*Missing Heading]]
"#,
    )
    .document();

    assert_eq!(doc.targets.len(), 2);
    assert_eq!(doc.diagnostics.len(), 4);
    assert!(doc
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("ambiguous")));
    assert_eq!(
        doc.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message.contains("was not found"))
            .count(),
        3
    );

    let links = doc
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
    assert!(links
        .iter()
        .any(|link| matches!(link.target, LinkTarget::Unresolved(_))));
}
