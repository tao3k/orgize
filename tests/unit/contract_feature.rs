//! Focused interval-index checks for the Scheme-AOT contract executor.

use super::{
    ContractFieldMatch, ContractQueryRule, ContractRelation, descendant_intervals, field_matches,
    in_intervals,
};
use crate::org_aot::parse_org_aot;

#[test]
fn descendant_ranges_merge_nested_targets_without_including_uncovered_targets() {
    let ends = [8, 5, 4, 4, 5, 8, 7, 8];
    let ranges = descendant_intervals(&[2, 1, 5], &ends);
    assert_eq!(ranges, vec![(2, 5), (6, 8)]);
    assert!(!in_intervals(1, &ranges));
    assert!(in_intervals(2, &ranges));
    assert!(!in_intervals(5, &ranges));
    assert!(in_intervals(7, &ranges));
}

#[test]
fn field_query_uses_the_shared_element_property_semantics() {
    let document = parse_org_aot("#+TITLE: Review plan\n").expect("Scheme-AOT fixture");
    let record = document
        .records()
        .iter()
        .find(|record| record.kind == "keyword")
        .expect("keyword Element");
    let query = ContractQueryRule {
        node_kind: "keyword",
        field_name: Some("value"),
        field_value: Some("Review plan"),
        field_match: ContractFieldMatch::Exact,
        relation: ContractRelation::Any,
        target_scope: false,
        target_binding: None,
    };
    assert!(field_matches(&document, record, query));
    assert!(!field_matches(
        &document,
        record,
        ContractQueryRule {
            field_value: Some("missing"),
            ..query
        }
    ));
    assert!(field_matches(
        &document,
        record,
        ContractQueryRule {
            field_value: Some("Review"),
            field_match: ContractFieldMatch::Contains,
            ..query
        }
    ));
}
