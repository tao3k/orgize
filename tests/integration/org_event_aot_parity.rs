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

#[test]
fn unclosed_blocks_recover_before_headlines_and_parent_boundaries() {
    for source in [
        "* Parent\n#+begin_src rust\nbody\n** Next\nvisible\n",
        "* Parent\n#+begin_quote\nbody\n** Next\nvisible\n",
        "#+begin_quote\n#+begin_src rust\nbody\n#+end_quote\nafter\n",
    ] {
        let structural = orgize::org_aot::parse_org_aot(source)
            .expect("structural parser recovers the unclosed block");
        let events = orgize::org_aot::parse_org_event_aot(source)
            .expect("Scheme event parser recovers the unclosed block");
        assert_eq!(events.syntax().to_string(), source);
        assert_eq!(events.records(), structural.records(), "source: {source}");
    }
}
