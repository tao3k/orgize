//! Compare the Scheme event entrypoint with the legacy structural fixture oracle.

use gerbil_parser_rowan::{GraphRecord, parse_structural_lines, project_syntax_graph};
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

fn structural_backbone(records: &[GraphRecord]) -> Vec<GraphRecord> {
    let retained = records
        .iter()
        .filter(|record| {
            !matches!(
                record.kind,
                "bold" | "italic" | "underline" | "strike-through"
            )
        })
        .collect::<Vec<_>>();
    let mut new_ids = vec![None; records.len()];
    for (new_id, record) in retained.iter().enumerate() {
        new_ids[record.id] = Some(new_id);
    }
    retained
        .into_iter()
        .enumerate()
        .map(|(new_id, record)| {
            let mut record = record.clone();
            record.id = new_id;
            record.parent_id = record.parent_id.and_then(|id| new_ids[id]);
            record.child_ids.retain(|id| new_ids[*id].is_some());
            for child_id in &mut record.child_ids {
                *child_id = new_ids[*child_id].expect("retained child id");
            }
            if record.kind == "src-block" {
                // The fixture oracle predates Scheme-owned structured header fields.
                // Their values have dedicated event/graph assertions; compare the
                // original lossless raw header here.
                record
                    .fields
                    .retain(|field| !matches!(field.name, "header-key" | "header-value"));
            }
            record
        })
        .collect()
}

#[test]
fn tracked_org_fixtures_preserve_structural_backbone_with_new_scheme_objects() {
    for source in [
        include_str!("../fixtures/org-elements/dynamic-block.org"),
        include_str!("../fixtures/org-elements/customer-queries.org"),
        include_str!(
            "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
        ),
    ] {
        let structural = structural_records(source);
        let events = parse_org_aot(source).expect("Scheme event algorithm parses the fixture");
        assert_eq!(events.syntax().to_string(), source);
        let mut projected = structural_backbone(events.records());
        assert_eq!(projected.len(), structural.len());
        for (event, legacy) in projected.iter_mut().zip(&structural) {
            if event.kind == "paragraph"
                && event.range.start() == legacy.range.start()
                && event.range.end() >= legacy.range.end()
            {
                let post_blank = source
                    .get(usize::from(legacy.range.end())..usize::from(event.range.end()))
                    .expect("paragraph ranges are source boundaries");
                assert!(
                    post_blank.trim().is_empty(),
                    "only Org post-blank may differ"
                );
                event.range = legacy.range;
            }
        }
        assert_eq!(projected, structural);
    }
}

#[test]
fn representative_fixture_keeps_the_new_footnote_shape_and_source() {
    let source = include_str!("../fixtures/org-elements/representative.org");
    let events = parse_org_aot(source).expect("Scheme event algorithm parses the fixture");
    assert_eq!(events.syntax().to_string(), source);
    assert!(
        events
            .records()
            .iter()
            .any(|record| record.kind == "footnote-definition")
    );
    assert!(
        events
            .records()
            .iter()
            .any(|record| record.kind == "footnote-reference")
    );
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
