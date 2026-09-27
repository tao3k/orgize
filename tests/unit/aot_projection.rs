//! Public Scheme AOT parser and owned Element projection admissions.

use crate::{
    Org,
    ast::{ElementData, ObjectData},
};

#[test]
fn public_parser_projects_quoted_emphasis_from_scheme_events() {
    for (source, kind) in [(r#""*quoted*""#, "bold"), ("'/quoted/'", "italic")] {
        let parsed = Org::parse(source);
        assert_eq!(parsed.to_org(), source);
        assert_eq!(
            parsed
                .records()
                .iter()
                .filter(|record| record.kind == kind)
                .count(),
            1
        );
    }
}

#[test]
fn aot_table_rows_and_cells_reach_the_owned_ast() {
    let source = "* H\n| Principle ID | Evidence |\n|--------------+----------|\n| P-001        | one      |\n";
    let parsed = Org::parse(source);
    let document = parsed.document();
    let table = document.sections[0]
        .children
        .iter()
        .find_map(|element| match &element.data {
            ElementData::Table(table) => Some(table),
            _ => None,
        })
        .expect("AOT table in section");
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0].cells.len(), 2);
    assert_eq!(table.rows[2].cells.len(), 2);
    assert_eq!(
        table.rows[0].cells[0].objects[0].data,
        ObjectData::Plain("Principle ID".to_owned())
    );
}

#[test]
fn named_source_block_uses_scheme_affiliation_in_owned_ast() {
    let document = Org::parse(
        "* Contract\n#+NAME: task.rule\n#+BEGIN_SRC org-contract\n(assert exists (headline))\n#+END_SRC\n",
    )
    .document();
    let blocks = document.source_block_records();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].name.as_deref(), Some("task.rule"));
}
