//! Scheme-declared ordinary Org drawer through the generic Rowan block engine.

use gerbil_parser_rowan::{SyntaxNode, SyntaxToken};

fn parse(source: &str) -> SyntaxNode {
    orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-declared Org drawer parses")
        .syntax()
}

fn name(node: &SyntaxNode) -> &'static str {
    orgize::org_aot::org_language_spec().kinds[usize::from(node.kind().0)].name
}

fn token_name(token: &SyntaxToken) -> &'static str {
    orgize::org_aot::org_language_spec().kinds[usize::from(token.kind().0)].name
}

#[test]
fn scheme_declared_named_drawer_preserves_name_and_nested_elements() {
    let source = "* Task\n:NOTE:  \nfirst\n[[https://example.test][link]]\n:END:\n** Next\n";
    let document = orgize::org_aot::parse_org_aot(source).unwrap();
    let drawers: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "drawer")
        .collect();
    assert_eq!(drawers.len(), 1);
    assert_eq!(drawers[0].field("name"), Some("NOTE"));
    let root = document.syntax();
    assert_eq!(root.to_string(), source);
    let drawer = root
        .descendants()
        .find(|node| name(node) == "OrgDrawer")
        .unwrap();
    assert_eq!(
        drawer
            .descendants()
            .filter(|node| name(node) == "OrgParagraph")
            .count(),
        1
    );
    assert_eq!(
        drawer
            .descendants()
            .filter(|node| name(node) == "OrgLink")
            .count(),
        1
    );
    let name_token = drawer
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token_name(token) == "DrawerName")
        .unwrap();
    let range = name_token.text_range();
    assert_eq!(
        &source[usize::from(range.start())..usize::from(range.end())],
        "NOTE"
    );
}

#[test]
fn named_drawer_recovery_does_not_swallow_following_headline() {
    let source = "* Task\n:NOTE:\nbody\n** Next\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgDrawer")
            .count(),
        0
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        2
    );
}

#[test]
fn malformed_named_drawer_openers_remain_text() {
    for source in [":END:\n", ":9BAD:\n", ":BAD: trailing\n"] {
        let root = parse(source);
        assert_eq!(root.to_string(), source);
        assert_eq!(
            root.descendants()
                .filter(|node| name(node) == "OrgDrawer")
                .count(),
            0
        );
    }
}
