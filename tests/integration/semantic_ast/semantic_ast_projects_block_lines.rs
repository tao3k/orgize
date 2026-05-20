use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{BlockKind, ElementData},
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/source-example-block-lines.org");

#[test]
fn semantic_ast_projects_source_and_example_block_lines() {
    let doc = Org::parse(SOURCE).document();

    assert_clean_projection(&doc);

    let blocks = doc
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Block(block) => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].kind, BlockKind::Source);
    assert_eq!(blocks[0].lines.len(), 3);
    assert_eq!(blocks[0].lines[0].number, 1);
    assert_eq!(blocks[0].lines[0].source, ",* not a headline");
    assert_eq!(blocks[0].lines[0].value, "* not a headline");
    assert_eq!(blocks[0].lines[0].line_ending.as_deref(), Some("\n"));
    assert_eq!(blocks[0].lines[0].ann.start.line, 3);
    assert_eq!(blocks[0].lines[0].ann.start.column, 1);
    assert_eq!(
        blocks[0].lines[1].code_ref,
        Some(blocks[0].code_refs[0].clone())
    );
    assert_eq!(blocks[0].lines[1].ann.start.line, 4);

    assert_eq!(blocks[1].kind, BlockKind::Example);
    assert_eq!(blocks[1].lines.len(), 1);
    assert_eq!(
        blocks[1].lines[0].code_ref,
        Some(blocks[1].code_refs[0].clone())
    );

    let bare = doc.to_bare();
    let bare_blocks = bare
        .children
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Block(block) => Some(block),
            _ => None,
        })
        .collect::<Vec<_>>();

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_block_lines", bare_blocks);
    });
}
