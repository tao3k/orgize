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
    parsed.syntax()
}

fn name(node: &SyntaxNode) -> &'static str {
    grammar::LANGUAGE.kinds[usize::from(node.kind().0)].name
}

#[test]
fn headings_form_nested_sections_and_blocks_mask_headlines() {
    let source = "é\r\n* Parent\n#+BEGIN_SRC rust\n** code, not a headline\n#+END_SRC\n** Child\nbody\r* Sibling\n";
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
fn unclosed_source_block_recovers_losslessly_at_eof() {
    let source = "* Open\r\n#+begin_src rust\r\n** source text\r\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        1
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
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
