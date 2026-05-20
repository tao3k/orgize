use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ElementData, TableColumnAlignment},
};

#[test]
fn semantic_ast_projects_table_column_alignment_metadata() {
    let doc = Org::parse(
        r#"| <l> | <c> | <r> |
| Name | Count | Note |
|------+-------+------|
| Foo  |     1 | ok   |

| <10> | <r3> |
| Text |    2 |
"#,
    )
    .document();

    assert_clean_projection(&doc);

    let tables = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Table(table) => Some(table),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(tables.len(), 2);

    assert_eq!(
        tables[0].column_alignments,
        vec![
            Some(TableColumnAlignment::Left),
            Some(TableColumnAlignment::Center),
            Some(TableColumnAlignment::Right),
        ]
    );
    assert_eq!(tables[0].rows.len(), 4);
    assert_eq!(tables[0].rows[0].cells.len(), 3);
    assert!(format!("{:?}", tables[0].rows[0].cells[0].objects).contains("<l>"));

    assert_eq!(
        tables[1].column_alignments,
        vec![None, Some(TableColumnAlignment::Right)]
    );
    assert_eq!(tables[1].rows.len(), 2);
    assert!(format!("{:?}", tables[1].rows[0].cells[0].objects).contains("<10>"));
}
