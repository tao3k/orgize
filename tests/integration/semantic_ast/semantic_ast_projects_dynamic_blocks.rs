use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{DynamicBlockContentState, DynamicBlockWriterKind, ElementData},
    org_aot::parse_org_aot,
};

#[test]
fn semantic_ast_projects_dynamic_block_registry_records_supported_and_unknown_writers() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 1
| Headline | Time |
#+END:

* Column area
#+BEGIN: columnview :id local :format "%ITEM %TODO"
| ITEM | TODO |
#+END:

#+BEGIN: custom :foo bar :phrase "hello :world" :columns ITEM TODO :flag
#+END:
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let ElementData::Block(parsed) = &doc.children[0].data else {
        panic!("first element must be the Scheme-classified dynamic block");
    };
    assert_eq!(
        parsed.parameters.as_deref(),
        Some(":scope file :maxlevel 1")
    );
    assert_eq!(parsed.header_args.len(), 2);

    let records = doc.dynamic_block_records();
    assert_eq!(records.len(), 3);

    let clocktable = &records[0];
    assert_eq!(clocktable.name, "clocktable");
    assert_eq!(clocktable.writer, DynamicBlockWriterKind::ClockTable);
    assert_eq!(clocktable.parameters.len(), 2);
    assert_eq!(clocktable.parameters[0].key, "scope");
    assert_eq!(clocktable.parameters[0].value.as_deref(), Some("file"));
    assert_eq!(
        clocktable.content_state,
        DynamicBlockContentState::ExistingOutput
    );
    assert_eq!(clocktable.content_line_count, 1);

    let columnview = &records[1];
    assert_eq!(columnview.name, "columnview");
    assert_eq!(columnview.writer, DynamicBlockWriterKind::ColumnView);
    assert_eq!(columnview.parameters[0].key, "id");
    assert_eq!(columnview.parameters[0].value.as_deref(), Some("local"));
    assert_eq!(columnview.parameters[1].key, "format");
    assert_eq!(
        columnview.parameters[1].value.as_deref(),
        Some("\"%ITEM %TODO\"")
    );
    assert_eq!(
        columnview.content_state,
        DynamicBlockContentState::ExistingOutput
    );

    let custom = &records[2];
    assert_eq!(custom.name, "custom");
    assert_eq!(custom.writer, DynamicBlockWriterKind::Unknown);
    assert_eq!(custom.parameters[0].key, "foo");
    assert_eq!(custom.parameters[0].value.as_deref(), Some("bar"));
    assert_eq!(custom.parameters[1].key, "phrase");
    assert_eq!(
        custom.parameters[1].value.as_deref(),
        Some("\"hello :world\"")
    );
    assert_eq!(custom.parameters[1].raw, ":phrase \"hello :world\"");
    assert_eq!(custom.parameters[2].key, "columns");
    assert_eq!(custom.parameters[2].value.as_deref(), Some("ITEM TODO"));
    assert_eq!(custom.parameters[3].key, "flag");
    assert_eq!(custom.parameters[3].value, None);
    assert_eq!(custom.content_state, DynamicBlockContentState::Empty);
    assert_eq!(custom.content_line_count, 0);
}

#[test]
fn dynamic_writer_name_is_not_replaced_by_an_affiliated_name() {
    let source = "#+NAME: report\n#+BEGIN: clocktable :scope file\n#+END:\n";
    let graph = parse_org_aot(source).expect("Scheme-AOT graph");
    let dynamic = graph
        .records()
        .iter()
        .find(|record| record.kind == "dynamic-block")
        .expect("Scheme-classified dynamic block");
    assert_eq!(dynamic.field("name"), Some("clocktable"));

    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);

    let records = doc.dynamic_block_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].name, "clocktable");
    assert_eq!(records[0].writer, DynamicBlockWriterKind::ClockTable);
    assert_eq!(doc.clock_table_plans().len(), 1);
}

#[test]
fn dynamic_block_content_uses_scheme_closing_range() {
    let source = "#+BEGIN: clocktable\r\n\r\n#+END:later\r\n  output\r\n#+END:\r\n";
    let graph = parse_org_aot(source).expect("Scheme-AOT graph");
    let dynamic = graph
        .records()
        .iter()
        .find(|record| record.kind == "dynamic-block")
        .expect("Scheme-classified dynamic block");
    let end = dynamic
        .field_range("end")
        .expect("Scheme closing-line field");
    assert_eq!(
        &source[usize::from(end.start())..usize::from(end.end())],
        "#+END:\r\n"
    );

    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.dynamic_block_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].content_line_count, 3);
    assert_eq!(
        records[0].content_state,
        DynamicBlockContentState::ExistingOutput
    );
}
