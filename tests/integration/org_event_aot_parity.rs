//! Exact Element graph parity on the Git-tracked representative Org fixture.

#[test]
fn tracked_org_fixtures_have_exact_event_element_graph_parity() {
    for source in [
        include_str!("../fixtures/org-elements/representative.org"),
        include_str!("../fixtures/org-elements/dynamic-block.org"),
        include_str!("../fixtures/org-elements/customer-queries.org"),
        include_str!(
            "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
        ),
    ] {
        let structural = orgize::org_aot::parse_org_aot(source)
            .expect("structural Scheme declaration parses the fixture");
        let events = orgize::org_aot::parse_org_event_aot(source)
            .expect("Scheme event algorithm parses the fixture");
        assert_eq!(structural.syntax().to_string(), source);
        assert_eq!(events.syntax().to_string(), source);
        assert_eq!(events.records(), structural.records());
    }
}
