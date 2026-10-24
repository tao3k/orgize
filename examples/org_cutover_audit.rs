//! Read-only coverage inventory for the public Scheme AOT Org parser.

use std::collections::{BTreeMap, BTreeSet};

use orgize::Org;
use orgize::org_aot::{org_graph_spec, org_language_spec};
#[rustfmt::skip]
#[path = "../languages/org/generated/elements.rs"]
mod elements;

fn generated_kinds(root: &gerbil_parser_runtime::SyntaxNode) -> BTreeMap<&'static str, usize> {
    let mut kinds = BTreeMap::new();
    for node in root.descendants() {
        let kind = org_language_spec().kinds[usize::from(node.kind().0)].name;
        *kinds.entry(kind).or_insert(0) += 1;
    }
    kinds
}

fn main() {
    // SAFETY: standalone entrypoint, before workers or children and host I/O.
    unsafe { orgize::initialize_native_runtime() }.expect("native startup");
    let source = include_str!("../tests/fixtures/org-elements/representative.org");
    let document = Org::parse(source);
    let event_root = document.syntax();

    assert_eq!(event_root.to_string(), source);
    assert_eq!(
        document.receipt().grammar_digest,
        org_language_spec().grammar_digest
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
    let event_kinds = generated_kinds(&event_root);
    println!(
        "public Scheme AOT node kinds in fixture ({}): {event_kinds:#?}",
        event_kinds.len()
    );
    let event_record_counts =
        document
            .records()
            .iter()
            .fold(BTreeMap::new(), |mut counts, record| {
                *counts.entry(record.kind).or_insert(0usize) += 1;
                counts
            });
    println!("Scheme event-AOT Element kinds: {event_record_counts:#?}");
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
    println!("status: public parser uses Scheme AOT; catalog projection gaps above remain open");
}
