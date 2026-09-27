use orgize::{
    Org, ParseConfig, TextRange,
    ast::{Checkbox, ElementData, ListType, ObjectData, TimestampKind},
};

#[test]
fn owned_projection_uses_scheme_graph_objects_and_list_structure() {
    let doc = Org::parse(
        "* TODO Review :work:\nSee [[https://example.org][site]] at <2026-05-19 Tue>.\n- [X] done with src_rust{ok}\n",
    )
    .document();

    assert!(doc.diagnostics.is_empty());
    let section = &doc.sections[0];
    assert_eq!(section.raw_title, "Review");
    assert_eq!(
        section.todo.as_ref().map(|todo| todo.name.as_str()),
        Some("TODO")
    );

    let paragraph = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .expect("paragraph from AOT graph");
    let link = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Link(link) => Some(link),
            _ => None,
        })
        .expect("link from AOT graph");
    assert_eq!(link.path(), "https://example.org");
    assert!(link.has_description());
    assert!(matches!(&link.description[0].data, ObjectData::Plain(text) if text == "site"));
    let timestamp = paragraph
        .iter()
        .find_map(|object| match &object.data {
            ObjectData::Timestamp(timestamp) => Some(timestamp),
            _ => None,
        })
        .expect("timestamp from AOT graph");
    assert_eq!(timestamp.kind, TimestampKind::Active);
    assert_eq!(timestamp.raw, "<2026-05-19 Tue>");

    let list = section
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::List(list) => Some(list),
            _ => None,
        })
        .expect("list from AOT graph");
    assert_eq!(list.list_type, ListType::Unordered);
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].bullet, "-");
    assert_eq!(list.items[0].checkbox, Some(Checkbox::On));
    let item_paragraph = list.items[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Paragraph(objects) => Some(objects),
            _ => None,
        })
        .expect("item paragraph from AOT graph");
    assert!(item_paragraph.iter().any(|object| matches!(&object.data, ObjectData::InlineSrc { language, value, .. } if language == "rust" && value == "ok")));
}

#[test]
fn file_todo_profile_reverts_to_caller_config_after_source_edit() {
    let source = "#+TODO: WAIT(w) | DONE(d)\n* WAIT Task\n";
    let config = ParseConfig {
        todo_keywords: (vec!["NEXT".into()], vec!["FINISHED".into()]),
        ..ParseConfig::default()
    };
    let mut doc = config.parse(source);
    assert_eq!(doc.config().todo_keywords.0, ["WAIT"]);
    assert_eq!(doc.config().todo_keywords.1, ["DONE"]);

    let directive_end = source.find('\n').expect("directive line") + 1;
    doc.replace_range(TextRange::new(0.into(), (directive_end as u32).into()), "");
    assert_eq!(doc.config().todo_keywords.0, ["NEXT"]);
    assert_eq!(doc.config().todo_keywords.1, ["FINISHED"]);
    assert_eq!(
        doc.headlines()
            .next()
            .and_then(|headline| headline.todo_keyword()),
        None
    );
}
