//! Source-backed graph and recovery checks for the sole Scheme event parser.

#[path = "../support/org_graph.rs"]
mod graph_support;
pub(crate) use graph_support::assert_graph_integrity;
use orgize::org_aot::parse_org_aot;

fn tracked_org_fixtures_keep_source_and_element_ancestry() {
    for (name, source, required) in [
        (
            "aot_dynamic_block_graph",
            include_str!("../fixtures/org-elements/dynamic-block.org"),
            &["headline", "dynamic-block", "link", "paragraph"][..],
        ),
        (
            "aot_customer_queries_graph",
            include_str!("../fixtures/org-elements/customer-queries.org"),
            &["keyword", "headline", "property-drawer", "src-block"],
        ),
        (
            "aot_contract_scope_graph",
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
            ),
            &["headline", "property-drawer", "node-property", "link"],
        ),
    ] {
        let document = parse_org_aot(source).expect("Scheme event parser accepts tracked Org");
        assert_eq!(document.syntax().to_string(), source);
        assert_graph_integrity(source, document.records());
        for kind in required {
            assert!(
                document.records().iter().any(|record| record.kind == *kind),
                "missing {kind} in tracked fixture"
            );
        }
        insta::assert_debug_snapshot!(name, document.records());
    }
}

fn representative_fixture_keeps_footnote_ancestry_and_source() {
    let source = include_str!("../fixtures/org-elements/representative.org");
    let document = parse_org_aot(source).expect("Scheme event parser accepts representative Org");
    assert_eq!(document.syntax().to_string(), source);
    assert_graph_integrity(source, document.records());
    for kind in ["footnote-definition", "footnote-reference", "timestamp"] {
        assert!(document.records().iter().any(|record| record.kind == kind));
    }
    insta::assert_debug_snapshot!("aot_representative_graph", document.records());
}

fn unclosed_blocks_recover_before_headlines_and_parent_boundaries() {
    for (source, expected_headlines) in [
        ("* Parent\n#+begin_src rust\nbody\n** Next\nvisible\n", 2),
        ("* Parent\n#+begin_quote\nbody\n** Next\nvisible\n", 2),
        (
            "#+begin_quote\n#+begin_src rust\nbody\n#+end_quote\nafter\n",
            0,
        ),
        (
            "* Parent\n:PROPERTIES:\n:ID: one\nmalformed\n:END:\n** Next\n",
            2,
        ),
    ] {
        let document = parse_org_aot(source).expect("Scheme event parser recovers Org source");
        assert_eq!(document.syntax().to_string(), source);
        assert_graph_integrity(source, document.records());
        assert_eq!(
            document
                .records()
                .iter()
                .filter(|record| record.kind == "headline")
                .count(),
            expected_headlines
        );
        if expected_headlines == 2 {
            assert!(document.records().iter().any(|record| {
                record.kind == "headline" && record.field("title") == Some("Next")
            }));
        }
    }
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "org_event_aot_parity::tracked_org_fixtures_keep_source_and_element_ancestry",
        tracked_org_fixtures_keep_source_and_element_ancestry,
    ),
    (
        "org_event_aot_parity::representative_fixture_keeps_footnote_ancestry_and_source",
        representative_fixture_keeps_footnote_ancestry_and_source,
    ),
    (
        "org_event_aot_parity::unclosed_blocks_recover_before_headlines_and_parent_boundaries",
        unclosed_blocks_recover_before_headlines_and_parent_boundaries,
    ),
];
