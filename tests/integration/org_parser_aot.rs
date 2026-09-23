//! Scheme POO declaration -> gerbil-parser AOT table -> contextual Rowan CST.

#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/structure.rs"]
mod structure;

use gerbil_parser_rowan::SyntaxNode;

fn parse(source: &str) -> SyntaxNode {
    let parsed = gerbil_parser_rowan::parse_structural_lines(
        &grammar::LANGUAGE,
        &structure::STRUCTURE,
        source,
    )
    .unwrap_or_else(|error| panic!("Org structural AOT rejected source: {error:?}"));
    assert_eq!(parsed.receipt().language, "org");
    assert_eq!(
        parsed.receipt().grammar_digest,
        grammar::LANGUAGE.grammar_digest
    );
    assert_eq!(
        parsed.receipt().parser_digest,
        Some(structure::STRUCTURE.parser_digest)
    );
    parsed.syntax()
}

fn name(node: &SyntaxNode) -> &'static str {
    grammar::LANGUAGE.kinds[usize::from(node.kind().0)].name
}

fn token_name(token: &gerbil_parser_rowan::SyntaxToken) -> &'static str {
    grammar::LANGUAGE.kinds[usize::from(token.kind().0)].name
}

#[test]
fn headings_form_nested_sections_and_closed_blocks_remain_lossless() {
    let source = "é\r\n* Parent\n#+BEGIN_SRC rust\ncode\n#+END_SRC\n** Child\nbody\r* Sibling\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(name(&root), "OrgFile");
    let sections: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgSection")
        .collect();
    assert_eq!(sections.len(), 3);
    assert_eq!(name(&sections[0].parent().unwrap()), "OrgFile");
    assert_eq!(name(&sections[1].parent().unwrap()), "OrgSection");
    assert_eq!(name(&sections[2].parent().unwrap()), "OrgFile");
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        3
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
    );
}

#[test]
fn unclosed_source_block_recovers_as_text_before_the_next_headline() {
    let source = "* Open\r\n#+begin_src rust\r\n** source text\r\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        2
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        0
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgTextLine")
            .count(),
        1
    );
}

#[test]
fn a_heading_bounds_source_block_recovery_even_when_an_end_marker_follows() {
    let source = "* One\n#+begin_src rust\n** Next\n#+end_src\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        2
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        0
    );
}

#[test]
fn structural_artifact_must_match_the_same_grammar_digest() {
    let mut stale = structure::STRUCTURE;
    stale.grammar_digest = "sha256:0000000000000000000000000000000000000000000000000000000000";
    let error = gerbil_parser_rowan::parse_structural_lines(&grammar::LANGUAGE, &stale, "")
        .expect_err("a stale POO projection cannot be silently used");
    assert_eq!(error.diagnostic.reason_kind, "invalid-structural-aot");
    assert_eq!(
        error.receipt.grammar_digest,
        grammar::LANGUAGE.grammar_digest
    );
    assert_eq!(
        error.receipt.parser_digest,
        Some(structure::STRUCTURE.parser_digest)
    );
}

#[test]
fn many_sibling_sections_keep_exact_source_order() {
    let source = "* item\n".repeat(10_000);
    let root = parse(&source);
    assert_eq!(root.to_string(), source);
    assert_eq!(root.children().count(), 10_000);
}

#[test]
fn git_tracked_org_document_is_lossless_at_the_current_structural_boundary() {
    let source = include_str!("../fixtures/org-elements/representative.org");
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSection")
            .count(),
        4
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
    );
}

#[test]
fn contract_scope_mvp_inputs_expose_drawers_and_node_properties() {
    let fixtures = [
        (
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/contracts.org"
            ),
            8,
            20,
            4,
            0,
        ),
        (
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
            ),
            4,
            6,
            0,
            6,
        ),
    ];
    for (source, drawers, properties, source_blocks, contract_org_properties) in fixtures {
        let root = parse(source);
        assert_eq!(root.to_string(), source);
        assert_eq!(
            root.descendants()
                .filter(|node| name(node) == "OrgHeadline")
                .count(),
            8
        );
        assert_eq!(
            root.descendants()
                .filter(|node| name(node) == "OrgPropertyDrawer")
                .count(),
            drawers
        );
        assert_eq!(
            root.descendants()
                .filter(|node| name(node) == "OrgSourceBlock")
                .count(),
            source_blocks
        );
        let property_nodes: Vec<_> = root
            .descendants()
            .filter(|node| name(node) == "OrgNodeProperty")
            .collect();
        assert_eq!(property_nodes.len(), properties);
        let mut contract_org_count = 0;
        for node in property_nodes {
            let range = node.text_range();
            let original = &source[usize::from(range.start())..usize::from(range.end())];
            assert_eq!(node.to_string(), original);
            assert!(original.trim_start().starts_with(':'));
            let tokens: Vec<_> = node
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .collect();
            let key = tokens
                .iter()
                .find(|token| token_name(token) == "PropertyKey")
                .expect("every admitted node-property has a typed key");
            let value = tokens
                .iter()
                .find(|token| token_name(token) == "PropertyValue")
                .expect("fixture node-properties have typed values");
            for token in [key, value] {
                let range = token.text_range();
                assert_eq!(
                    &source[usize::from(range.start())..usize::from(range.end())],
                    token.text()
                );
            }
            contract_org_count += usize::from(key.text() == "CONTRACT_ORG");
        }
        assert_eq!(contract_org_count, contract_org_properties);
    }
}

#[test]
fn invalid_property_drawer_recovers_without_claiming_node_properties() {
    let source = "* Task\n:PROPERTIES:\nnot a property\n:END:\n** Next\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgPropertyDrawer")
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
