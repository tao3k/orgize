//! Scheme-authored event fixtures reach Rowan and Org Element projection.
//! Fixture parity does not mean production parsing has switched to event AOT.

use gerbil_parser_rowan::{KindCategory, TreeEvent, parse_generated_events, project_syntax_graph};
use orgize::org_aot::{org_graph_spec, org_language_spec};

#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/line-events.rs"]
mod generated_line_events;

const HANDOFF_TEST_DIGEST: &str =
    "sha256:8b41c0fcb53588c81a44b83ec9e530bdb6125096637cba4934c96f7c2965abd6";

#[test]
fn scheme_authored_line_algorithm_aot_builds_lossless_rowan() {
    let source = "* α\r\nbody\n";
    let events = generated_line_events::parse_org_line_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_line_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme-generated Org line events satisfy Rowan");

    assert_eq!(parsed.syntax().to_string(), source);
    assert_eq!(
        parsed.receipt().parser_digest,
        Some(generated_line_events::PARSER_DIGEST)
    );
    let kinds: Vec<_> = parsed
        .syntax()
        .children()
        .map(|node| org_language_spec().kinds[usize::from(node.kind().0)].name)
        .collect();
    assert_eq!(kinds, ["OrgHeadline", "OrgTextLine"]);
}

fn kind(name: &str, category: KindCategory) -> u16 {
    org_language_spec()
        .kinds
        .iter()
        .position(|candidate| candidate.name == name && candidate.category == category)
        .and_then(|index| u16::try_from(index).ok())
        .expect("Scheme grammar declares the requested syntax kind")
}

#[test]
fn executable_scheme_outline_events_reach_rowan_and_element_projection() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../languages/org/v1/generated/outline-event-fixture.json"
    ))
    .expect("Scheme outline fixture is valid JSON");
    let source = fixture["source"].as_str().expect("fixture source");
    let events: Vec<_> = fixture["events"]
        .as_array()
        .expect("fixture events")
        .iter()
        .map(|event| {
            let fields = event.as_array().expect("event tuple");
            match fields[0].as_str().expect("event tag") {
                "start" => TreeEvent::StartNode(kind(
                    fields[1].as_str().expect("node kind"),
                    KindCategory::Node,
                )),
                "token" => TreeEvent::Token {
                    kind: kind(fields[1].as_str().expect("token kind"), KindCategory::Token),
                    start: usize::try_from(fields[2].as_u64().expect("token start"))
                        .expect("start fits usize"),
                    end: usize::try_from(fields[3].as_u64().expect("token end"))
                        .expect("end fits usize"),
                },
                "finish" => TreeEvent::FinishNode,
                tag => panic!("unknown Scheme event tag: {tag}"),
            }
        })
        .collect();
    let parsed = parse_generated_events(org_language_spec(), HANDOFF_TEST_DIGEST, source, &events)
        .expect("Scheme events satisfy Rowan's source and nesting contract");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme events support Org Element projection");
    let headline_titles: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "headline")
        .filter_map(|record| record.field("title"))
        .collect();
    assert_eq!(headline_titles, ["Parent", "Child"]);
    let todo = records
        .iter()
        .find(|record| record.kind == "keyword")
        .expect("Scheme keyword projects as Element");
    assert_eq!(todo.field("key"), Some("TODO"));
    assert_eq!(todo.field("value"), Some("TODO | DONE"));
    let source_block = records
        .iter()
        .find(|record| record.kind == "src-block")
        .expect("Scheme source block projects as Element");
    assert_eq!(source_block.field("language"), Some("rust"));
    assert_eq!(source_block.field("body"), Some("α\n"));
    let paragraph = parsed
        .syntax()
        .descendants()
        .find(|node| node.kind().0 == kind("OrgParagraph", KindCategory::Node))
        .expect("Scheme events retain a paragraph boundary");
    assert_eq!(paragraph.to_string(), "summary\n");
    assert!(records.iter().any(|record| record.kind == "paragraph"));
    let table = records
        .iter()
        .find(|record| record.kind == "table")
        .expect("Scheme table projects as an Element");
    assert_eq!(
        usize::from(table.range.start()),
        source.find("| a | b |").unwrap()
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "table-row")
            .count(),
        2
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "table-rule-row")
            .count(),
        1
    );
    let cells: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "table-cell")
        .filter_map(|record| record.field("text"))
        .collect();
    assert_eq!(cells, [" a ", " b ", " c\\|d ", " α "]);
    let example = records
        .iter()
        .find(|record| record.kind == "example-block")
        .expect("Scheme example block projects as an Element");
    assert_eq!(example.field("body"), Some("| literal |\n"));
    let export = records
        .iter()
        .find(|record| record.kind == "export-block")
        .expect("Scheme export block projects as an Element");
    assert_eq!(export.field("backend"), Some("html"));
    assert_eq!(export.field("body"), Some("<b>α</b>\n"));
    let comment = records
        .iter()
        .find(|record| record.kind == "comment-block")
        .expect("Scheme comment block projects as an Element");
    assert_eq!(comment.field("body"), Some("ignored\n"));
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "table")
            .count(),
        1
    );
    let drawer = records
        .iter()
        .find(|record| record.kind == "property-drawer")
        .expect("Scheme property drawer projects as an Element");
    let properties: Vec<_> = records
        .iter()
        .filter(|record| record.parent_id == Some(drawer.id))
        .filter(|record| record.kind == "node-property")
        .map(|record| (record.field("key"), record.field("value")))
        .collect();
    assert_eq!(
        properties,
        [(Some("ID"), Some("alpha")), (Some("EMPTY"), None)]
    );
    let planning = records
        .iter()
        .find(|record| record.kind == "planning")
        .expect("Scheme headline-local planning projects as an Element");
    assert_eq!(planning.field("key"), Some("SCHEDULED"));
    assert_eq!(planning.field("value"), Some("<2026-01-01>"));
    let clock = records
        .iter()
        .find(|record| record.kind == "clock")
        .expect("Scheme CLOCK line projects as an Element");
    assert_eq!(clock.field("value"), Some("[a]--[b]"));
    let link = records
        .iter()
        .find(|record| record.kind == "link")
        .expect("Scheme inline link projects as an Object");
    assert_eq!(link.field("path"), Some("https://example.test"));
    assert_eq!(link.field("description"), Some("α"));
    let lists: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "plain-list")
        .collect();
    let items: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "item")
        .collect();
    assert_eq!(lists.len(), 3);
    assert_eq!(items.len(), 4);
    assert_eq!(items[0].field("bullet"), Some("-"));
    assert_eq!(items[1].field("bullet"), Some("-"));
    assert_eq!(items[2].field("bullet"), Some("-"));
    assert_eq!(items[3].field("bullet"), Some("-"));
    assert_eq!(lists[1].parent_id, Some(items[0].id));
    assert_eq!(items[1].parent_id, Some(lists[1].id));
    let quote = records
        .iter()
        .find(|record| record.kind == "quote-block")
        .expect("Scheme-owned container projects as a quote Element");
    assert_eq!(lists[2].parent_id, Some(quote.id));
    assert_eq!(items[3].parent_id, Some(lists[2].id));
    let dynamic = records
        .iter()
        .find(|record| record.kind == "dynamic-block")
        .expect("Scheme-owned dynamic block projects as an Element");
    assert_eq!(dynamic.field("name"), Some("note"));
    let logbook = records
        .iter()
        .find(|record| record.kind == "drawer")
        .expect("Scheme-owned named drawer projects as an Element");
    assert_eq!(logbook.field("name"), Some("LOGBOOK"));

    let transitional = orgize::org_aot::parse_org_aot(source)
        .expect("the current production parser accepts the handoff fixture");
    assert_eq!(records.len(), transitional.records().len());
    for (actual, expected) in records.iter().zip(transitional.records()) {
        assert_eq!(actual.parent_id, expected.parent_id);
        assert_eq!(actual.child_ids, expected.child_ids);
        assert_eq!(actual.kind, expected.kind);
        assert_eq!(actual.range, expected.range);
        assert_eq!(actual.fields, expected.fields);
    }
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
