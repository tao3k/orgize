use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{DynamicBlockContentState, DynamicBlockWriterKind},
    Org,
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

#+BEGIN: custom :foo bar
#+END:
"#,
    )
    .document();
    assert_clean_projection(&doc);

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
    assert_eq!(custom.content_state, DynamicBlockContentState::Empty);
    assert_eq!(custom.content_line_count, 0);
}
