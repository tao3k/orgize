use orgize::{
    ast::{AstRef, ElementData, MarkupKind, ObjectData, TodoState},
    Org,
};

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

    assert!(doc.diagnostics.is_empty());
    assert_eq!(doc.children.len(), 1);
    assert_eq!(doc.sections.len(), 1);

    let section = &doc.sections[0];
    assert_eq!(section.level, 1);
    assert_eq!(section.todo.as_ref().unwrap().state, TodoState::Todo);
    assert_eq!(section.raw_title, "Heading ");
    assert_eq!(section.tags, ["work"]);
    assert_eq!(section.anchor.as_deref(), Some("heading-id"));
    assert!(section.planning.scheduled.is_some());

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
    assert!(paragraph
        .iter()
        .any(|object| matches!(object.data, ObjectData::Link { .. })));

    insta::assert_debug_snapshot!("semantic_bare_ast", doc.to_bare());
}

#[test]
fn annotations_map_and_fold_work_across_the_tree() {
    let doc = Org::parse("* DONE A\nBody with /italic/ text.").document();

    assert_eq!(doc.ann.start.line, 1);
    assert_eq!(doc.ann.start.column, 1);
    assert_eq!(doc.sections[0].ann.start.line, 1);
    assert_eq!(doc.sections[0].children[0].ann.start.line, 2);

    let object_count = doc.fold(0usize, |count, node| match node {
        AstRef::Object(_) => count + 1,
        _ => count,
    });
    assert!(object_count >= 3);

    let ranges = doc.map_ann(|ann| ann.range);
    assert_eq!(ranges.ann, doc.ann.range);

    let bare = doc
        .try_map_ann(|_| Ok::<_, std::convert::Infallible>(()))
        .unwrap();
    assert_eq!(bare, doc.to_bare());
}

#[test]
fn semantic_ast_projects_inline_babel_and_footnote_details() {
    let doc = Org::parse(
        r#"call_square[:results output](4)[:results html] and src_rust[:exports code]{let x = 1;} and [fn:note:See *bold* text]."#,
    )
    .document();

    assert!(doc.diagnostics.is_empty());
    let paragraph = match &doc.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };

    let inline_call = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::InlineCall {
                name,
                arguments,
                header,
                end_header,
                ..
            } => Some((name, arguments, header, end_header)),
            _ => None,
        })
        .expect("inline call object");
    assert_eq!(inline_call.0, "square");
    assert_eq!(inline_call.1, "4");
    assert_eq!(inline_call.2.as_deref(), Some(":results output"));
    assert_eq!(inline_call.3.as_deref(), Some(":results html"));

    let inline_src = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::InlineSrc {
                language,
                parameters,
                value,
                ..
            } => Some((language, parameters, value)),
            _ => None,
        })
        .expect("inline src object");
    assert_eq!(inline_src.0, "rust");
    assert_eq!(inline_src.1.as_deref(), Some(":exports code"));
    assert_eq!(inline_src.2, "let x = 1;");

    let footnote = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::FootnoteRef { label, definition } => Some((label, definition)),
            _ => None,
        })
        .expect("footnote ref object");
    assert_eq!(footnote.0.as_deref(), Some("note"));
    assert!(footnote.1.iter().any(|object| matches!(
        object.data,
        ObjectData::Markup {
            kind: MarkupKind::Bold,
            ..
        }
    )));
}

#[test]
fn existing_html_traversal_still_uses_the_lossless_substrate() {
    let html = Org::parse(
        r#"* title
paragraph with [[https://example.com][link]]

- one
- two

#+begin_quote
quoted
#+end_quote
"#,
    )
    .to_html();

    insta::assert_snapshot!("semantic_ast_html_compatibility", html);
}
