//! Read-only differential inventory for the Org parser cutover.
//!
//! This intentionally reports mismatches instead of admitting the AOT parser
//! as a public replacement before element, object, and typed-AST parity.

use std::collections::{BTreeMap, BTreeSet};

use orgize::org_aot::{
    org_graph_spec, org_language_spec, org_structure_spec, parse_org_aot, parse_org_event_aot,
};
use orgize::{Org, SyntaxNode as PublicSyntaxNode, rowan::ast::AstNode};
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
        let kind = org_language_spec().kinds[usize::from(node.kind().0)].name;
        *kinds.entry(kind).or_insert(0) += 1;
    }
    kinds
}

fn main() {
    let source = include_str!("../tests/fixtures/org-elements/representative.org");
    let public = Org::parse(source);
    let public_root = public.syntax_document().syntax().clone();
    let generated =
        parse_org_aot(source).expect("generated Org structure should accept the pinned fixture");
    let generated_root = generated.syntax();
    let event_tree = parse_org_event_aot(source)
        .expect("Scheme AOT Org events should accept the pinned fixture");
    let event_root = event_tree.syntax();

    assert_eq!(public_root.to_string(), source);
    assert_eq!(generated_root.to_string(), source);
    assert_eq!(event_root.to_string(), source);
    assert_eq!(
        generated.receipt().grammar_digest,
        org_language_spec().grammar_digest
    );
    assert_eq!(
        generated.receipt().parser_digest,
        Some(org_structure_spec().parser_digest)
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
    let event_kinds = generated_kinds(&event_root);
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
        "Scheme event-AOT node kinds in fixture ({}): {event_kinds:#?}",
        event_kinds.len()
    );
    let event_record_counts =
        event_tree
            .records()
            .iter()
            .fold(BTreeMap::new(), |mut counts, record| {
                *counts.entry(record.kind).or_insert(0usize) += 1;
                counts
            });
    println!("Scheme event-AOT Element kinds: {event_record_counts:#?}");
    let structural_records = generated.records();
    let event_records = event_tree.records();
    if structural_records == event_records {
        println!("structural/event Element graph: exact record parity");
    } else {
        let first_difference = structural_records
            .iter()
            .zip(event_records)
            .position(|(structural, event)| structural != event)
            .unwrap_or(structural_records.len().min(event_records.len()));
        println!(
            "structural/event Element graph differs: structural={}, event={}, first_index={first_difference}",
            structural_records.len(),
            event_records.len()
        );
        println!(
            "first structural record: {:?}",
            structural_records.get(first_difference)
        );
        println!(
            "first event record: {:?}",
            event_records.get(first_difference)
        );
    }
    let projected: BTreeSet<_> = org_graph_spec()
        .rules
        .iter()
        .flat_map(|rule| [rule.kind, rule.category])
        .collect();
    let missing_elements: Vec<_> = elements::ORG_ELEMENT_KINDS
        .iter()
        .copied()
        .filter(|kind| !projected.contains(kind))
        .collect();
    let missing_objects: Vec<_> = elements::ORG_OBJECT_KINDS
        .iter()
        .copied()
        .filter(|kind| !projected.contains(kind))
        .collect();
    println!(
        "Scheme catalog not yet projected as typed Element kinds ({}): {missing_elements:?}",
        missing_elements.len()
    );
    println!(
        "Scheme catalog not yet projected as typed Object kinds ({}): {missing_objects:?}",
        missing_objects.len()
    );
    println!(
        "status: three-way source parity only; element/object and public-AST parity not admitted"
    );
}
