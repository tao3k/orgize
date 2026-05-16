use crate::semantic_ast::support::assert_clean_projection;
use orgize::{ast::ColumnViewScope, Org as OrgParser};

#[test]
fn semantic_ast_projects_document_and_section_column_views() {
    let doc = OrgParser::parse(
        r#"#+COLUMNS: %25ITEM(Task) %TODO %3PRIORITY %Effort{:}

* Project
:PROPERTIES:
:COLUMNS: %ITEM %Effort(Effort){:}
:END:
"#,
    )
    .document();

    assert_clean_projection(&doc);
    let records = doc.column_view_records();
    assert_eq!(records.len(), 2);

    assert_eq!(records[0].scope, ColumnViewScope::DocumentKeyword);
    assert_eq!(records[0].columns[0].property, "ITEM");
    assert_eq!(records[0].columns[0].width, Some(25));
    assert_eq!(records[0].columns[0].title.as_deref(), Some("Task"));
    assert_eq!(records[0].columns[3].summary_operator.as_deref(), Some(":"));

    match &records[1].scope {
        ColumnViewScope::SectionProperty {
            level,
            title,
            outline_path,
        } => {
            assert_eq!(*level, 1);
            assert_eq!(title, "Project");
            assert_eq!(outline_path, &["Project"]);
        }
        other => panic!("expected section-scoped COLUMNS property, got {other:#?}"),
    }
    assert_eq!(records[1].columns[1].title.as_deref(), Some("Effort"));
}
