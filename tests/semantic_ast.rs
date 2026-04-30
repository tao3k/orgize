use orgize::{
    ast::{AstMut, AstRef, ElementData, MarkupKind, ObjectData, TodoState},
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
fn traversal_visits_annotation_bearing_metadata_nodes() {
    let mut doc = Org::parse(
        r#"#+TITLE: Demo
#+AUTHOR: Author
* Heading
:PROPERTIES:
:CUSTOM_ID: heading-id
:END:
#+ATTR_HTML: :class compact
|   A |   B |
|-----+-----|
|   1 |   2 |

- tag :: item
"#,
    )
    .document();

    assert!(doc.diagnostics.is_empty());

    #[derive(Default)]
    struct Counts {
        keywords: usize,
        properties: usize,
        list_items: usize,
        table_rows: usize,
        table_cells: usize,
    }

    let counts = doc.fold(Counts::default(), |mut counts, node| {
        match node {
            AstRef::Keyword(_) => counts.keywords += 1,
            AstRef::Property(_) => counts.properties += 1,
            AstRef::ListItem(_) => counts.list_items += 1,
            AstRef::TableRow(_) => counts.table_rows += 1,
            AstRef::TableCell(_) => counts.table_cells += 1,
            _ => {}
        }
        counts
    });

    assert_eq!(counts.keywords, 3);
    assert_eq!(counts.properties, 1);
    assert_eq!(counts.list_items, 1);
    assert_eq!(counts.table_rows, 3);
    assert_eq!(counts.table_cells, 4);

    doc.visit_mut(|node| {
        if let AstMut::Keyword(keyword) = node {
            if keyword.key == "TITLE" {
                keyword.value = " Changed".into();
            }
        }
    });
    assert_eq!(
        doc.children
            .iter()
            .find_map(|element| match &element.data {
                ElementData::Keyword(keyword) if keyword.key == "TITLE" => Some(&keyword.value),
                _ => None,
            })
            .map(String::as_str),
        Some(" Changed")
    );
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
fn semantic_ast_projects_footnote_definition_label_and_body() {
    let doc = Org::parse("#+CAPTION: A note\n[fn:WORD-1] See *bold* text\n").document();

    assert!(doc.diagnostics.is_empty());
    let element = &doc.children[0];
    assert_eq!(element.affiliated_keywords[0].key, "CAPTION");
    assert_eq!(element.affiliated_keywords[0].value, " A note");

    let definition = match &element.data {
        ElementData::FootnoteDef(definition) => definition,
        _ => panic!("expected footnote definition"),
    };
    assert_eq!(definition.label, "WORD-1");

    let body = match &definition.children[0].data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected footnote body paragraph"),
    };
    assert!(body.iter().any(|object| matches!(
        &object.data,
        ObjectData::Plain(value) if value.contains("*bold*")
    )));
}

#[test]
fn semantic_ast_keeps_affiliated_keywords_out_of_paragraph_objects() {
    let doc = Org::parse("#+ATTR_HTML: :width 300px\n[[./img/a.jpg]]").document();

    assert!(doc.diagnostics.is_empty());
    let paragraph = &doc.children[0];
    assert_eq!(paragraph.affiliated_keywords.len(), 1);
    assert_eq!(paragraph.affiliated_keywords[0].key, "ATTR_HTML");
    assert_eq!(paragraph.affiliated_keywords[0].value, " :width 300px");

    let objects = match &paragraph.data {
        ElementData::Paragraph(objects) => objects,
        _ => panic!("expected paragraph"),
    };
    assert_eq!(objects.len(), 1);
    assert!(matches!(objects[0].data, ObjectData::Link { .. }));
}

#[test]
fn semantic_ast_projects_clean_clock_duration() {
    let doc = Org::parse("* Work\nCLOCK: [2003-09-16 Tue 09:39] =>  1:00\n").document();

    assert!(doc.diagnostics.is_empty());
    let clock = doc.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Clock(clock) => Some(clock),
            _ => None,
        })
        .expect("clock element");

    assert!(clock.value.is_some());
    assert_eq!(clock.duration.as_deref(), Some("1:00"));
    assert!(clock.raw.contains("=>  1:00"));
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
