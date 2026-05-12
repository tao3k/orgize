use crate::semantic_ast::support::assert_clean_projection;
use orgize::{ast::ElementData, Org};

#[test]
fn semantic_ast_projects_table_el() {
    let doc = Org::parse("  +---+\n  | a |\n  +---+\n").document();

    assert_clean_projection(&doc);
    match &doc.children[0].data {
        ElementData::TableEl { raw } => {
            assert!(raw.contains("| a |"));
            assert!(raw.starts_with("  +---+"));
        }
        other => panic!("expected table.el element, got {other:#?}"),
    }
}
