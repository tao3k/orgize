//! Org-owned dynamic block declarations through the generic Rowan executor.

use gerbil_parser_rowan::{SyntaxNode, SyntaxToken};

fn node_name(node: &SyntaxNode) -> &'static str {
    orgize::org_aot::org_language_spec().kinds[usize::from(node.kind().0)].name
}

fn token_name(token: &SyntaxToken) -> &'static str {
    orgize::org_aot::org_language_spec().kinds[usize::from(token.kind().0)].name
}

#[test]
fn scheme_declared_dynamic_block_exposes_name_header_and_nested_elements() {
    let source = include_str!("../fixtures/org-elements/dynamic-block.org");
    let document = orgize::org_aot::parse_org_aot(source).expect("dynamic block parses");
    assert_eq!(document.syntax().to_string(), source);
    let blocks: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "dynamic-block")
        .collect();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].field("name"), Some("clocktable"));
    assert!(
        blocks[0]
            .field("header")
            .is_some_and(|value| value.contains(":scope file"))
    );
    let block = document
        .syntax()
        .descendants()
        .find(|node| node_name(node) == "OrgDynamicBlock")
        .expect("dynamic block CST node");
    assert_eq!(
        block
            .descendants()
            .filter(|node| node_name(node) == "OrgParagraph")
            .count(),
        1
    );
    assert_eq!(
        block
            .descendants()
            .filter(|node| node_name(node) == "OrgLink")
            .count(),
        1
    );
    let name = block
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token_name(token) == "DynamicBlockName")
        .expect("typed dynamic name");
    let range = name.text_range();
    assert_eq!(
        &source[usize::from(range.start())..usize::from(range.end())],
        "clocktable"
    );
}

#[test]
fn malformed_and_unclosed_dynamic_blocks_recover_as_text() {
    for source in [
        "#+BEGIN:\n#+END:\n",
        "#+BEGIN: 9bad\n#+END:\n",
        "* First\n#+BEGIN: clocktable\nbody\n** Next\n",
    ] {
        let root = orgize::org_aot::parse_org_aot(source)
            .expect("malformed dynamic block remains lossless")
            .syntax();
        assert_eq!(root.to_string(), source);
        assert_eq!(
            root.descendants()
                .filter(|node| node_name(node) == "OrgDynamicBlock")
                .count(),
            0
        );
    }
}
