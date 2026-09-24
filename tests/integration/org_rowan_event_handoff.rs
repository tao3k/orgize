//! Boundary test: Org-shaped nesting reaches Rowan without structural scanning.
//! These events are a fixture, not a claim that Orgize generates them yet.

use gerbil_parser_rowan::{KindCategory, TreeEvent, parse_generated_events};
use orgize::org_aot::org_language_spec;

const HANDOFF_TEST_DIGEST: &str =
    "sha256:8b41c0fcb53588c81a44b83ec9e530bdb6125096637cba4934c96f7c2965abd6";

fn kind(name: &str, category: KindCategory) -> u16 {
    org_language_spec()
        .kinds
        .iter()
        .position(|candidate| candidate.name == name && candidate.category == category)
        .and_then(|index| u16::try_from(index).ok())
        .expect("Scheme grammar declares the requested syntax kind")
}

macro_rules! org_event {
    (start $name:literal) => {
        TreeEvent::StartNode(kind($name, KindCategory::Node))
    };
    (token $name:literal, $start:expr, $end:expr) => {
        TreeEvent::Token {
            kind: kind($name, KindCategory::Token),
            start: $start,
            end: $end,
        }
    };
    (finish) => {
        TreeEvent::FinishNode
    };
}

#[test]
fn nested_org_events_reach_rowan_without_a_structural_engine_rule() {
    let source = "* Parent\n#+begin_src rust\nfn main() {}\n#+end_src\n** Child\n";
    let block_start = source.find("#+begin_src").expect("source block opening");
    let body_start = source.find("fn main()").expect("source block body");
    let block_end = source.find("#+end_src").expect("source block closing");
    let child_start = source.find("** Child").expect("child headline");
    let events = [
        org_event!(start "OrgFile"),
        org_event!(start "OrgSection"),
        org_event!(start "OrgHeadline"),
        org_event!(token "HeadlineLine", 0, block_start),
        org_event!(finish),
        org_event!(start "OrgSourceBlock"),
        org_event!(token "BlockBeginLine", block_start, body_start),
        org_event!(token "TextLine", body_start, block_end),
        org_event!(token "BlockEndLine", block_end, child_start),
        org_event!(finish),
        org_event!(start "OrgSection"),
        org_event!(start "OrgHeadline"),
        org_event!(token "HeadlineLine", child_start, source.len()),
        org_event!(finish),
        org_event!(finish),
        org_event!(finish),
        org_event!(finish),
    ];
    let parsed = parse_generated_events(org_language_spec(), HANDOFF_TEST_DIGEST, source, &events)
        .expect("Org event stream satisfies the generic Rowan contract");

    assert_eq!(parsed.syntax().to_string(), source);
    assert_eq!(parsed.receipt().parser_digest, Some(HANDOFF_TEST_DIGEST));
    assert_eq!(
        parsed.selective_glr_receipt().winner_reason,
        "scheme-aot-events"
    );
    let sections: Vec<_> = parsed
        .syntax()
        .descendants()
        .filter(|node| node.kind().0 == kind("OrgSection", KindCategory::Node))
        .collect();
    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections[1]
            .ancestors()
            .filter(|node| node.kind().0 == kind("OrgSection", KindCategory::Node))
            .count(),
        2
    );
}
