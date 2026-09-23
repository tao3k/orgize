//! Read-only differential inventory for the Org parser cutover.
//!
//! This intentionally reports mismatches instead of admitting the AOT parser
//! as a public replacement before element, object, and typed-AST parity.

use std::collections::BTreeMap;

use orgize::{Org, SyntaxNode as PublicSyntaxNode, rowan::ast::AstNode};

#[rustfmt::skip]
#[path = "../languages/org/v1/generated/parser.rs"]
mod grammar;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/structure.rs"]
mod structure;
#[rustfmt::skip]
#[path = "../languages/org/v1/generated/elements.rs"]
mod elements;

fn public_kinds(root: &PublicSyntaxNode) -> BTreeMap<String, usize> {
    let mut kinds = BTreeMap::new();
    for node in root.descendants() {
        *kinds.entry(format!("{:?}", node.kind())).or_insert(0) += 1;
    }
    kinds
}

fn generated_kinds(root: &gerbil_parser_rowan::SyntaxNode) -> BTreeMap<&'static str, usize> {
    let mut kinds = BTreeMap::new();
    for node in root.descendants() {
        let kind = grammar::LANGUAGE.kinds[usize::from(node.kind().0)].name;
        *kinds.entry(kind).or_insert(0) += 1;
    }
    kinds
}

fn main() {
    let source = include_str!("../tests/fixtures/org-elements/representative.org");
    let public = Org::parse(source);
    let public_root = public.syntax_document().syntax().clone();
    let generated = gerbil_parser_rowan::parse_structural_lines(
        &grammar::LANGUAGE,
        &structure::STRUCTURE,
        source,
    )
    .expect("generated Org structure should accept the pinned fixture");
    let generated_root = generated.syntax();

    assert_eq!(public_root.to_string(), source);
    assert_eq!(generated_root.to_string(), source);
    assert_eq!(
        generated.receipt().grammar_digest,
        grammar::LANGUAGE.grammar_digest
    );

    println!(
        "inventory {}: {} elements, {} greater elements, {} objects, {} recursive objects, {} affiliated keywords, {} restriction owners, {} secondary values",
        elements::ELEMENTS_DIGEST,
        elements::ORG_ELEMENT_KINDS.len(),
        elements::ORG_GREATER_ELEMENT_KINDS.len(),
        elements::ORG_OBJECT_KINDS.len(),
        elements::ORG_RECURSIVE_OBJECT_KINDS.len(),
        elements::ORG_AFFILIATED_KEYWORDS.len(),
        elements::ORG_OBJECT_RESTRICTIONS.len(),
        elements::ORG_SECONDARY_VALUES.len()
    );
    let public_kinds = public_kinds(&public_root);
    let generated_kinds = generated_kinds(&generated_root);
    println!(
        "public node kinds in fixture ({}): {public_kinds:#?}",
        public_kinds.len()
    );
    println!(
        "generated node kinds in fixture ({}): {generated_kinds:#?}",
        generated_kinds.len()
    );
    println!(
        "status: structural source parity only; element/object and public-AST parity not admitted"
    );
}
