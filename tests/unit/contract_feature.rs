//! Focused interval-index checks for the Scheme-AOT contract executor.

use super::{
    ContractFieldMatch, ContractQueryRule, ContractRelation, descendant_intervals, field_matches,
    in_intervals,
};
use gerbil_parser_rowan::{GraphFieldValue, GraphRecord};
use rowan::{TextRange, TextSize};

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
fn field_query_matches_any_value_of_a_repeated_property() {
    let record = GraphRecord {
        id: 0,
        parent_id: None,
        child_ids: Vec::new(),
        syntax_kind: 0,
        category: "section",
        kind: "headline",
        range: TextRange::new(TextSize::from(0), TextSize::from(0)),
        fields: vec![
            GraphFieldValue {
                name: "tags",
                value: "work".into(),
            },
            GraphFieldValue {
                name: "tags",
                value: "urgent".into(),
            },
        ],
    };
    let query = ContractQueryRule {
        node_kind: "headline",
        field_name: Some("tags"),
        field_value: Some("urgent"),
        field_match: ContractFieldMatch::Exact,
        relation: ContractRelation::Any,
        target_scope: false,
        target_binding: None,
    };
    assert!(field_matches(&record, query));
    assert!(!field_matches(
        &record,
        ContractQueryRule {
            field_value: Some("missing"),
            ..query
        }
    ));
}
