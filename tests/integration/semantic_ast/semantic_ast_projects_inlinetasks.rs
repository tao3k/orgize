use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org, ParseConfig,
    ast::{AstRef, ElementData, MarkupKind, ObjectData, TodoState},
    syntax_ast::SyntaxInlinetask,
};

#[test]
fn semantic_ast_projects_closed_inlinetasks_with_body() {
    let doc = Org::parse(
        r#"Intro.

*************** TODO [#A] *Inline* task :work:
SCHEDULED: <2026-05-10 Sun>
:PROPERTIES:
:CUSTOM_ID: inline-task
:END:
Body with [[https://example.com][link]].
*************** END

* Outline
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].level, 1);

    let inlinetask = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Inlinetask(inlinetask) => Some(inlinetask),
            _ => None,
        })
        .expect("inlinetask element");

    assert_eq!(inlinetask.level, 15);
    assert_eq!(inlinetask.todo.as_ref().unwrap().name, "TODO");
    assert_eq!(inlinetask.todo.as_ref().unwrap().state, TodoState::Todo);
    assert_eq!(inlinetask.priority.raw_cookie(), Some("A"));
    assert_eq!(inlinetask.priority.effective_text(), "A");
    assert_eq!(inlinetask.raw_title, "*Inline* task ");
    assert_eq!(inlinetask.tags, ["work"]);
    assert!(matches!(
        &inlinetask.title[0].data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    ));
    assert!(inlinetask.planning.scheduled.is_some());
    assert_eq!(inlinetask.properties[0].key, "CUSTOM_ID");
    assert_eq!(inlinetask.properties[0].value, "inline-task");
    assert!(matches!(
        &inlinetask.children[0].data,
        ElementData::Paragraph(objects)
            if objects.iter().any(|object| matches!(object.data, ObjectData::Link(_)))
    ));
    assert_eq!(inlinetask.end.as_ref().unwrap().level, 15);
    assert_eq!(
        inlinetask.end.as_ref().unwrap().raw.trim(),
        "*************** END"
    );

    let counts = doc.fold((0, 0), |(tasks, ends), node| match node {
        AstRef::Inlinetask(_) => (tasks + 1, ends),
        AstRef::InlinetaskEnd(_) => (tasks, ends + 1),
        _ => (tasks, ends),
    });
    assert_eq!(counts, (1, 1));
}

#[test]
fn semantic_ast_keeps_unclosed_inlinetask_from_consuming_following_paragraph() {
    let doc = Org::parse("*************** Note\nAfter text.\n").document();

    assert_clean_projection(&doc);
    assert!(matches!(
        &doc.children[0].data,
        ElementData::Inlinetask(inlinetask)
            if inlinetask.raw_title == "Note"
                && inlinetask.children.is_empty()
                && inlinetask.end.is_none()
    ));
    assert!(matches!(
        &doc.children[1].data,
        ElementData::Paragraph(objects)
            if matches!(&objects[0].data, ObjectData::Plain(value) if value == "After text.\n")
    ));
}

#[test]
fn inlinetask_min_level_keeps_lower_star_headlines_in_the_outline() {
    let org = Org::parse("************** Outline\nBody.\n");
    let doc = org.document();

    assert_clean_projection(&doc);
    assert!(doc.children.is_empty());
    assert_eq!(doc.sections.len(), 1);
    assert_eq!(doc.sections[0].level, 14);
    assert!(org.first_node::<SyntaxInlinetask>().is_none());

    let doc = ParseConfig {
        inlinetask_min_level: 4,
        ..Default::default()
    }
    .parse("**** Inline\nBody.\n")
    .document();

    assert_clean_projection(&doc);
    assert!(matches!(
        &doc.children[0].data,
        ElementData::Inlinetask(inlinetask)
            if inlinetask.level == 4 && inlinetask.raw_title == "Inline"
    ));
}
