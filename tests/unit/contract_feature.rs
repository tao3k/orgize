//! Focused Scheme-AOT contract property checks.

use super::{ContractQueryRule, ContractRelation, field_matches};
use crate::org_aot::parse_org_aot;
use crate::org_element_query::{OrgElementFieldMatch, OrgElementPropertyRule};

#[test]
fn generated_consumers_bind_the_current_graph_identity() {
    let digest = crate::org_aot::org_graph_spec().projection_digest;
    let queries = crate::org_element_query::org_element_query_pack();
    assert!(!queries.rules.is_empty());
    assert_eq!(queries.graph_digest, digest, "regenerate Scheme query pack");
    let contracts = crate::org_aot::org_contract_pack();
    assert!(!contracts.rules.is_empty());
    for rule in contracts.rules {
        assert_eq!(
            rule.graph_digest, digest,
            "regenerate Scheme contract {}",
            rule.id
        );
    }
}

pub(crate) const NATIVE_CASES: &[(&str, fn())] = &[(
    "field_query_uses_the_shared_element_property_semantics",
    field_query_uses_the_shared_element_property_semantics,
)];

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
