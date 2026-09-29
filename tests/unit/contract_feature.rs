//! Focused Scheme-AOT contract property checks.

use super::{ContractQueryRule, ContractRelation, field_matches};
use crate::org_aot::parse_org_aot;
use crate::org_element_query::{OrgElementFieldMatch, OrgElementPropertyRule};

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
        groups: &[&[OrgElementPropertyRule {
            name: "value",
            value: "Review plan",
            matcher: OrgElementFieldMatch::Exact,
        }]],
        relation: ContractRelation::Any,
        target_scope: false,
        target_binding: None,
    };
    assert!(field_matches(&document, record, query));
    assert!(!field_matches(
        &document,
        record,
        ContractQueryRule {
            groups: &[&[OrgElementPropertyRule {
                name: "value",
                value: "missing",
                matcher: OrgElementFieldMatch::Exact,
            }]],
            ..query
        }
    ));
    assert!(field_matches(
        &document,
        record,
        ContractQueryRule {
            groups: &[&[OrgElementPropertyRule {
                name: "value",
                value: "Review",
                matcher: OrgElementFieldMatch::Contains,
            }]],
            ..query
        }
    ));
}
