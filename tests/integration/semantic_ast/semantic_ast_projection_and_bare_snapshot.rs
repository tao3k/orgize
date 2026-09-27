use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ElementData, MarkupKind, ObjectData, TodoState},
};

#[test]
fn semantic_ast_projects_headline_title_objects_from_the_aot_graph() {
    let doc = Org::parse("* TODO *Bold* title :work:\n").document();
    let section = &doc.sections[0];
    assert_eq!(section.raw_title, "*Bold* title ");
    assert!(section.title.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}

#[test]
fn semantic_ast_inherits_headline_tags_from_the_aot_graph() {
    let doc = Org::parse("* Parent :work:\n** Child :urgent:\n").document();
    let parent = &doc.sections[0];
    let child = &parent.subsections[0];
    assert_eq!(parent.tags, ["work"]);
    assert_eq!(child.tags, ["urgent"]);
    assert_eq!(child.effective_tags, ["work", "urgent"]);
}

#[test]
fn semantic_ast_projection_and_bare_snapshot() {
    let doc = Org::parse(
        r#"#+TITLE: Demo
* TODO Heading :work:
SCHEDULED: <2026-04-30 Thu>
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
Paragraph with *bold*, [[https://example.com][a link]], and <2026-04-30 Thu>.

- [X] item one
- tag :: item two

#+begin_src rust
fn main() {}
#+end_src
"#,
    )
    .document();

    assert_clean_projection(&doc);
    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.sections.len(), 1);

    let section = &doc.sections[0];
    assert_eq!(section.level, 1);
    assert_eq!(section.todo.as_ref().unwrap().state, TodoState::Todo);
    assert_eq!(section.raw_title, "Heading ");
    assert_eq!(section.tags, ["work"]);
    assert_eq!(section.anchor.as_deref(), Some("heading-id"));
    let scheduled = section.planning.scheduled.as_ref().unwrap();
    assert_eq!(scheduled.start.as_ref().unwrap().year, 2026);
    assert_eq!(scheduled.start.as_ref().unwrap().month, 4);
    assert_eq!(scheduled.start.as_ref().unwrap().day, 30);
    assert_eq!(
        scheduled.start.as_ref().unwrap().day_name.as_deref(),
        Some("Thu")
    );

    let paragraph = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .expect("paragraph element");
    assert!(paragraph.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
    assert!(
        paragraph
            .iter()
            .any(|object| matches!(object.data, ObjectData::Link(_)))
    );

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_bare_ast", doc.to_bare());
    });
}
