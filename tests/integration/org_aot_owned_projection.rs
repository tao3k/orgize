#[test]
fn explicit_startup_precedes_parallel_org_aot_owned_projection_cases() {
    // SAFETY: initialize before creating application workers or children;
    // these parser cases do not use Scheme-owned I/O or subprocesses.
    unsafe { orgize::initialize_native_runtime() }.expect("native test startup");
    std::thread::scope(|scope| {
        scope.spawn(owned_projection_uses_scheme_graph_objects_and_list_structure);
        scope.spawn(file_todo_profile_reverts_to_caller_config_after_source_edit);
        scope.spawn(scheme_priority_cookie_reprojects_after_source_edit);
        scope.spawn(owned_footnote_definition_uses_scheme_graph_label_and_body);
        scope.spawn(keyword_graph_keeps_semantic_and_source_faithful_values_separate);
        scope.spawn(owned_drawer_and_babel_call_use_scheme_element_children);
        scope.spawn(owned_inlinetask_uses_scheme_begin_body_and_end_records);
    });
    println!("startup-native suite=org_aot_owned_projection concurrent-cases=7 complete OK");
}

use orgize::{
    Org, ParseConfig, TextRange,
    ast::{Checkbox, ElementData, ListType, ObjectData, TimestampKind},
};

fn owned_projection_uses_scheme_graph_objects_and_list_structure() {
    let doc = Org::parse(
        "* TODO Review :work:\nSee [[https://example.org][site]] at <2026-05-19 Tue>.\n- [X] done with src_rust{ok}\n",
    )
    .document();

    assert!(doc.diagnostics.is_empty());
    let section = &doc.sections[0];
    // raw_title preserves the source separator before the headline tag suffix.
    assert_eq!(section.raw_title, "Review ");
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
    // The bullet includes its exact source separator, independently of export formatting.
    assert_eq!(list.items[0].bullet, "- ");
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

fn scheme_priority_cookie_reprojects_after_source_edit() {
    let mut doc = Org::parse("* TODO [#A] Task\n");
    let headline_id = doc.headlines().next().expect("headline").id();
    assert_eq!(
        doc.headline_priority_cookie(headline_id).as_deref(),
        Some("A")
    );
    assert_eq!(doc.document().sections[0].priority.raw_cookie(), Some("A"));

    doc.replace_range(TextRange::new(7.into(), 11.into()), "[#64]");
    let headline_id = doc.headlines().next().expect("edited headline").id();
    assert_eq!(
        doc.headline_priority_cookie(headline_id).as_deref(),
        Some("64")
    );
    assert_eq!(doc.document().sections[0].priority.raw_cookie(), Some("64"));
}

fn owned_footnote_definition_uses_scheme_graph_label_and_body() {
    let doc = Org::parse("[fn:WORD-1] See *bold* text\n").document();
    assert!(doc.diagnostics.is_empty());
    let definition = match &doc.children[0].data {
        ElementData::FootnoteDef(definition) => definition,
        other => panic!("expected footnote definition, got {other:?}"),
    };
    assert_eq!(definition.label, "WORD-1");
    assert!(matches!(
        &definition.children[0].data,
        ElementData::Paragraph(objects) if !objects.is_empty()
    ));
}

fn keyword_graph_keeps_semantic_and_source_faithful_values_separate() {
    for (source, expected_raw) in [
        ("#+CAPTION:  A note\n[fn:note] Body\n", "  A note"),
        ("#+CAPTION:A note\n[fn:note] Body\n", "A note"),
    ] {
        let parsed = Org::parse(source);
        let keyword = parsed
            .records()
            .iter()
            .find(|record| record.kind == "keyword")
            .expect("caption keyword");
        assert_eq!(keyword.field("value"), Some("A note"));
        assert_eq!(keyword.field("raw-value"), Some(expected_raw));
        assert_eq!(
            parsed.document().children[0].affiliated_keywords[0].value,
            expected_raw
        );
    }
}

fn owned_drawer_and_babel_call_use_scheme_element_children() {
    let doc = Org::parse("#+CALL: name()\n:LOGBOOK:\nInside\n:END:\n").document();
    assert!(doc.diagnostics.is_empty());
    assert!(doc.children.iter().any(|element| matches!(
        &element.data,
        ElementData::BabelCall(call) if call.key == "CALL" && call.value == " name()"
    )));
    let drawer = doc
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Drawer(drawer) => Some(drawer),
            _ => None,
        })
        .expect("Scheme graph drawer");
    assert_eq!(drawer.name, "LOGBOOK");
    assert!(matches!(
        &drawer.children[0].data,
        ElementData::Paragraph(_)
    ));
}

fn owned_inlinetask_uses_scheme_begin_body_and_end_records() {
    let source = "*************** TODO [#A] Inline\nBody\n*************** END\n";
    let parsed = Org::parse(source);
    assert_eq!(parsed.to_org(), source);
    let doc = parsed.document();
    assert!(doc.diagnostics.is_empty());
    let inlinetask = match &doc.children[0].data {
        ElementData::Inlinetask(inlinetask) => inlinetask,
        other => panic!("expected inlinetask, got {other:?}"),
    };
    assert_eq!(inlinetask.level, 15);
    assert_eq!(
        inlinetask.todo.as_ref().map(|todo| todo.name.as_str()),
        Some("TODO")
    );
    assert_eq!(inlinetask.priority.raw_cookie(), Some("A"));
    assert!(matches!(
        &inlinetask.children[0].data,
        ElementData::Paragraph(_)
    ));
    assert_eq!(inlinetask.end.as_ref().map(|end| end.level), Some(15));
    assert_eq!(
        inlinetask.end.as_ref().map(|end| end.raw.as_str()),
        Some("*************** END\n")
    );
}
