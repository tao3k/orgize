//! Scheme event-AOT contract projection at the public Org AOT boundary.

#[test]
fn scheme_event_aot_evaluates_the_document_headline_contract() {
    let source = include_str!(
        "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
    );
    let event = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme event AOT projects through the contract path");
    let contract = orgize::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "document.headlines.v1")
        .expect("Scheme AOT pack includes the document contract");
    let scope = orgize::contract_feature::ContractScopeNodeId(0);
    let actual = event
        .evaluate_contract(contract, scope)
        .expect("Scheme event AOT evaluates contract");
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].assertion_id, "document.has-headline");
    assert_eq!(actual[0].matched_count, 3);
    assert!(actual[0].passed);
}
