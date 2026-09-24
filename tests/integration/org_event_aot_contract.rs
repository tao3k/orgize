//! Scheme event-AOT contract projection parity at the public Org AOT boundary.

#[test]
fn scheme_event_aot_evaluates_the_same_element_contract() {
    let source = include_str!(
        "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
    );
    let structural = orgize::org_aot::parse_org_aot(source).expect("structural baseline parses");
    let event = orgize::org_aot::parse_org_event_aot(source)
        .expect("Scheme event AOT projects through the contract path");
    let contract = orgize::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "document.headlines.v1")
        .expect("Scheme AOT pack includes the document contract");
    let scope = orgize::contract_feature::ContractScopeNodeId(0);
    let expected = structural
        .evaluate_contract(contract, scope)
        .expect("structural baseline evaluates contract");
    let actual = event
        .evaluate_contract(contract, scope)
        .expect("Scheme event AOT evaluates contract");
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(&expected) {
        assert_eq!(actual.assertion_id, expected.assertion_id);
        assert_eq!(actual.matched_count, expected.matched_count);
        assert_eq!(actual.passed, expected.passed);
    }
}
