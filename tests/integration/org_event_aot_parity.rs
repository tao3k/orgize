//! Compare the Scheme event entrypoint with the legacy structural fixture oracle.

use gerbil_parser_rowan::{parse_structural_lines, project_syntax_graph};
use orgize::org_aot::{org_graph_spec, org_language_spec, parse_org_aot};

fn structural_records(source: &str) -> Vec<gerbil_parser_rowan::GraphRecord> {
    let parsed = parse_structural_lines(
        org_language_spec(),
        &crate::org_structural_fixture::STRUCTURE,
        source,
    )
    .expect("legacy structural fixture oracle accepts the source");
    project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("legacy structural fixture oracle projects Elements")
}

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
        let structural = structural_records(source);
        let events = parse_org_aot(source).expect("Scheme event algorithm parses the fixture");
        assert_eq!(events.syntax().to_string(), source);
        assert_eq!(events.records(), structural);
    }
}

#[test]
fn unclosed_blocks_recover_before_headlines_and_parent_boundaries() {
    for source in [
        "* Parent\n#+begin_src rust\nbody\n** Next\nvisible\n",
        "* Parent\n#+begin_quote\nbody\n** Next\nvisible\n",
        "#+begin_quote\n#+begin_src rust\nbody\n#+end_quote\nafter\n",
        "* Parent\n:PROPERTIES:\n:ID: one\nmalformed\n:END:\n** Next\n",
    ] {
        let structural = structural_records(source);
        let events =
            parse_org_aot(source).expect("Scheme event parser recovers the unclosed block");
        assert_eq!(events.syntax().to_string(), source);
        assert_eq!(events.records(), structural, "source: {source}");
    }
}
