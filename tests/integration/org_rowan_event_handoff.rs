//! Scheme-authored event fixtures reach Rowan and Org Element projection.
//! Fixture parity does not mean production parsing has switched to event AOT.

use gerbil_parser_rowan::{KindCategory, TreeEvent, parse_generated_events, project_syntax_graph};
use orgize::org_aot::{org_graph_spec, org_language_spec};

#[rustfmt::skip]
#[path = "../../languages/org/v1/generated/line-events.rs"]
mod generated_line_events;

#[rustfmt::skip]
mod generated_context_events {
    use gerbil_parser_rowan::TreeEvent;
    include!(concat!(env!("OUT_DIR"), "/org_rowan_events.rs"));
}

const HANDOFF_TEST_DIGEST: &str =
    "sha256:8b41c0fcb53588c81a44b83ec9e530bdb6125096637cba4934c96f7c2965abd6";

#[test]
fn org_scheme_event_aot_projects_grouped_comments_and_nested_scope() {
    let source = "# first\n# second\ntext\n#+begin_quote\n# nested\n#+end_quote\n#\n";
    let document = orgize::org_aot::parse_org_event_aot(source)
        .expect("Scheme comment strategy builds a lossless Rowan document");
    assert_eq!(document.syntax().to_string(), source);
    let comments: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "comment")
        .collect();
    assert_eq!(comments.len(), 3);
    assert_eq!(
        comments[0].values("source-line").collect::<Vec<_>>(),
        ["# first\n", "# second\n"]
    );
    assert_eq!(comments[1].field("source-line"), Some("# nested\n"));
    assert_eq!(comments[2].field("source-line"), Some("#\n"));
    let quote = document
        .records()
        .iter()
        .find(|record| record.kind == "quote-block")
        .expect("recursive quote block is an Element");
    assert_eq!(comments[1].parent_id, Some(quote.id));

    let listed = orgize::org_aot::parse_org_event_aot("- item\n  # child\n")
        .expect("indented comment stays inside its list item");
    let item = listed
        .records()
        .iter()
        .find(|record| record.kind == "item")
        .expect("list item projects as an Element");
    let child = listed
        .records()
        .iter()
        .find(|record| record.kind == "comment")
        .expect("comment projects inside the list item");
    assert_eq!(child.parent_id, Some(item.id));
}

#[test]
fn org_scheme_event_aot_projects_diary_sexp_without_claiming_percent_text() {
    let source = "%%(diary-anniversary 1 1 2000)\n%%not-diary\n";
    let document = orgize::org_aot::parse_org_event_aot(source)
        .expect("Scheme diary-sexp declaration reaches Rowan and Elements");
    assert_eq!(document.syntax().to_string(), source);
    let diary = document
        .records()
        .iter()
        .find(|record| record.kind == "diary-sexp")
        .expect("diary-sexp is a typed Element");
    assert_eq!(diary.field("value"), Some("%%(diary-anniversary 1 1 2000)"));
    assert_eq!(diary.range.start(), 0u32.into());
    assert_eq!(diary.range.end(), 31u32.into());
    assert!(
        document
            .records()
            .iter()
            .any(|record| record.kind == "paragraph" && record.range.start() == 31u32.into())
    );

    let trailing = orgize::org_aot::parse_org_event_aot("%%(x) \r\n")
        .expect("CRLF and trailing spaces remain lossless");
    assert_eq!(trailing.syntax().to_string(), "%%(x) \r\n");
    let value = trailing
        .records()
        .iter()
        .find(|record| record.kind == "diary-sexp")
        .and_then(|record| record.field("value"));
    assert_eq!(value, Some("%%(x) "));
}

#[test]
fn org_scheme_event_aot_projects_inline_code_and_verbatim_values() {
    let source = "a ~code~ =verb= [[id:x]] z\n";
    let document = orgize::org_aot::parse_org_event_aot(source)
        .expect("Scheme inline Object strategy builds a lossless Rowan document");
    assert_eq!(document.syntax().to_string(), source);
    let code = document
        .records()
        .iter()
        .find(|record| record.kind == "code")
        .expect("code is a typed Object");
    assert_eq!(code.field("value"), Some("code"));
    let verbatim = document
        .records()
        .iter()
        .find(|record| record.kind == "verbatim")
        .expect("verbatim is a typed Object");
    assert_eq!(verbatim.field("value"), Some("verb"));
    assert!(
        document
            .records()
            .iter()
            .any(|record| record.kind == "link")
    );

    let negative = orgize::org_aot::parse_org_event_aot("x~y~ ~unclosed\n")
        .expect("invalid and unclosed markup remains source text");
    assert_eq!(negative.syntax().to_string(), "x~y~ ~unclosed\n");
    assert!(
        !negative
            .records()
            .iter()
            .any(|record| record.kind == "code" || record.kind == "verbatim")
    );
}

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

#[test]
fn org_scheme_context_algorithm_aot_masks_headlines_inside_source_blocks() {
    let source = "* Parent\n#+BeGiN_SrC rust\n** fake\n#+EnD_SrC\n** Child\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Org Scheme context algorithm builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let kinds: Vec<_> = parsed
        .syntax()
        .descendants()
        .map(|node| org_language_spec().kinds[usize::from(node.kind().0)].name)
        .collect();
    assert_eq!(
        kinds,
        [
            "OrgFile",
            "OrgSection",
            "OrgHeadline",
            "OrgSourceBlock",
            "OrgSection",
            "OrgHeadline",
        ]
    );
    assert_eq!(
        parsed.receipt().parser_digest,
        Some(generated_context_events::PARSER_DIGEST)
    );
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme source-block fields project through Org Elements");
    let block = records
        .iter()
        .find(|record| record.kind == "src-block")
        .expect("source block is an Element");
    assert_eq!(block.field("language"), Some("rust"));
    assert_eq!(block.field("body"), Some("** fake\n"));
}

#[test]
fn org_scheme_context_algorithm_aot_projects_paragraph_elements() {
    let source = "alpha\nβ\n \t\nnext\n* H\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme paragraph transitions build a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme paragraphs project through the Org Element graph");
    let paragraphs: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "paragraph")
        .collect();
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].range.start(), 0u32.into());
    assert_eq!(paragraphs[0].range.end(), 9u32.into());
    assert_eq!(paragraphs[1].range.start(), 12u32.into());
    assert_eq!(paragraphs[1].range.end(), 17u32.into());
}

#[test]
fn org_scheme_context_algorithm_aot_projects_horizontal_rules() {
    let source = "before\n-----\nafter\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme horizontal-rule events build a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("horizontal rule projects through the Org Element graph");
    let kinds: Vec<_> = records.iter().map(|record| record.kind).collect();
    assert_eq!(
        kinds,
        ["org-data", "paragraph", "horizontal-rule", "paragraph"]
    );
    let rule = records
        .iter()
        .find(|record| record.kind == "horizontal-rule")
        .expect("horizontal rule is a typed Element");
    assert_eq!(rule.range.start(), 7u32.into());
    assert_eq!(rule.range.end(), 13u32.into());

    let near_misses = "----\n----- x\n";
    let events = generated_context_events::parse_org_rowan_events(near_misses);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        near_misses,
        &events,
    )
    .expect("non-rules remain lossless paragraphs");
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("non-rules project through the Org Element graph");
    assert!(
        !records
            .iter()
            .any(|record| record.kind == "horizontal-rule")
    );
}

#[test]
fn org_scheme_context_algorithm_aot_groups_fixed_width_lines() {
    let source = "first\n: A\n:\n: B\nlast\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme fixed-width events build a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("fixed-width text projects through the Org Element graph");
    let kinds: Vec<_> = records.iter().map(|record| record.kind).collect();
    assert_eq!(kinds, ["org-data", "paragraph", "fixed-width", "paragraph"]);
    let fixed = records
        .iter()
        .find(|record| record.kind == "fixed-width")
        .expect("fixed-width is a typed Element");
    assert_eq!(fixed.range.start(), 6u32.into());
    assert_eq!(fixed.range.end(), 16u32.into());

    let nested = "#+begin_quote\n: A\n#+end_quote\n:PROPERTIES:\n:ID: x\n:END:\n";
    let events = generated_context_events::parse_org_rowan_events(nested);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        nested,
        &events,
    )
    .expect("fixed-width container closure and following properties stay lossless");
    assert_eq!(parsed.syntax().to_string(), nested);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("nested fixed-width text and property drawer project");
    let kinds: Vec<_> = records.iter().map(|record| record.kind).collect();
    assert!(kinds.contains(&"quote-block"));
    assert!(kinds.contains(&"fixed-width"));
    assert!(kinds.contains(&"property-drawer"));
}

#[test]
fn org_scheme_context_algorithm_projects_dynamic_keywords_for_todo_queries() {
    let source = "#+SEQ_TODO: TODO | DONE \r\n* TODO Work\n#+CALL: name()\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme key-line algorithm builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme keyed lines project through Org Elements");
    let keyword = records
        .iter()
        .find(|record| record.kind == "keyword")
        .expect("file-local TODO declaration is a keyword Element");
    assert_eq!(keyword.field("key"), Some("SEQ_TODO"));
    assert_eq!(keyword.field("value"), Some("TODO | DONE"));
    let headline = records
        .iter()
        .find(|record| record.kind == "headline")
        .expect("headline is an Element with source-backed fields");
    assert_eq!(headline.field("markers"), Some("*"));
    assert_eq!(headline.field("title"), Some("TODO Work"));
    let babel_call = records
        .iter()
        .find(|record| record.kind == "babel-call")
        .expect("CALL is a distinct Babel Element");
    assert_eq!(babel_call.field("key"), Some("CALL"));
    assert_eq!(babel_call.field("value"), Some("name()"));
}

#[test]
fn org_scheme_context_algorithm_projects_declared_planning_and_clock() {
    let source = "* H\nSCHEDULED: now\nCLOCK: 2\n* N\nDEADLINE: x\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme planning and clock algorithm builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("declared key lines project through Org Elements");
    let planning: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "planning")
        .collect();
    assert_eq!(planning.len(), 2);
    assert_eq!(planning[0].field("key"), Some("SCHEDULED"));
    assert_eq!(planning[0].field("value"), Some("now"));
    assert_eq!(planning[1].field("key"), Some("DEADLINE"));
    assert_eq!(planning[1].field("value"), Some("x"));
    let clock = records
        .iter()
        .find(|record| record.kind == "clock")
        .expect("clock Element");
    assert_eq!(clock.field("value"), Some("2"));

    let empty_source = "* H\nSCHEDULED:  \nCLOCK:  \n";
    let empty_events = generated_context_events::parse_org_rowan_events(empty_source);
    let empty_tree = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        empty_source,
        &empty_events,
    )
    .expect("empty declared values retain ordered source spans");
    assert_eq!(empty_tree.syntax().to_string(), empty_source);
}

#[test]
fn org_scheme_context_algorithm_projects_escaped_tables_and_rule_rows() {
    let source = "* H\n| a\\|b | c |\n|---+---|\nplain\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme table algorithm builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme table rows project through Org Elements");
    let table = records
        .iter()
        .find(|record| record.kind == "table")
        .expect("one table Element");
    let rows: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "table-row")
        .collect();
    assert_eq!(rows.len(), 1);
    assert!(rows.iter().all(|row| row.parent_id == Some(table.id)));
    let rule_rows: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "table-rule-row")
        .collect();
    assert_eq!(rule_rows.len(), 1);
    assert_eq!(rule_rows[0].parent_id, Some(table.id));
    let cells: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "table-cell")
        .collect();
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0].field("text"), Some(" a\\|b "));
    assert_eq!(cells[1].field("text"), Some(" c "));
    assert!(cells.iter().all(|cell| cell.parent_id == Some(rows[0].id)));
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "paragraph")
            .count(),
        1
    );

    let even_escape_source = "| a\\\\|b | c |\n";
    let even_escape_events = generated_context_events::parse_org_rowan_events(even_escape_source);
    let even_escape_tree = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        even_escape_source,
        &even_escape_events,
    )
    .expect("even backslash parity exposes the table separator");
    let even_escape_records = project_syntax_graph(
        org_language_spec(),
        org_graph_spec(),
        &even_escape_tree.syntax(),
    )
    .expect("even parity table cells project");
    assert_eq!(
        even_escape_records
            .iter()
            .filter(|record| record.kind == "table-cell")
            .count(),
        3
    );
}

#[test]
fn org_scheme_context_algorithm_projects_headline_property_drawers() {
    let source = "* H\n:PROPERTIES:\n:ID: alpha\n:END:\nbody\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme drawer algorithm builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme property drawer projects through Org Elements");
    let headline = records
        .iter()
        .find(|record| record.kind == "headline")
        .expect("owner headline");
    let drawer = records
        .iter()
        .find(|record| record.kind == "property-drawer")
        .expect("property drawer Element");
    let property = records
        .iter()
        .find(|record| record.kind == "node-property")
        .expect("typed node-property Element");
    assert_eq!(drawer.parent_id, Some(headline.id));
    assert_eq!(property.parent_id, Some(drawer.id));
    assert_eq!(property.field("key"), Some("ID"));
    assert_eq!(property.field("value"), Some("alpha"));
}

#[test]
fn org_scheme_context_algorithm_rejects_longer_block_marker_lookalikes() {
    let source = "#+begin_srcx\n* H\n:PROPERTIES:x\n:PROPERTIES:\n:ID: alpha\n:END: tail\n:END:\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("bounded Scheme markers preserve the source");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("bounded marker tree projects Elements");
    assert!(!records.iter().any(|record| record.kind == "src-block"));
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "property-drawer")
            .count(),
        1
    );
    let properties: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "node-property")
        .map(|record| (record.field("key"), record.field("value")))
        .collect();
    assert_eq!(
        properties,
        [(Some("ID"), Some("alpha")), (Some("END"), Some("tail"))]
    );
}

#[test]
fn org_scheme_context_algorithm_projects_poo_declared_opaque_blocks() {
    let source = "#+BEGIN_SRC rust\n** fake\n#+END_SRC\n#+begin_example\n* hidden\n#+end_example\n#+begin_comment\n| x |\n#+end_comment\n#+begin_export html\n<b>x</b>\n#+end_export\n* Visible\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme opaque-block strategy builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme opaque-block strategy projects Org Elements");
    for kind in [
        "src-block",
        "example-block",
        "comment-block",
        "export-block",
    ] {
        assert_eq!(
            records.iter().filter(|record| record.kind == kind).count(),
            1
        );
    }
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "headline")
            .count(),
        1,
        "headline and table-looking block bodies remain opaque"
    );
    let export = records
        .iter()
        .find(|record| record.kind == "export-block")
        .expect("export block Element");
    assert_eq!(export.field("backend"), Some("html"));
    assert_eq!(export.field("body"), Some("<b>x</b>\n"));
}

#[test]
fn org_scheme_context_algorithm_aot_projects_nested_lists() {
    let source = "- a\n  - b\n- c\n\n1. d\n2) e\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme list strategy builds a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);

    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme nested lists project Org Elements");
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "plain-list")
            .count(),
        3
    );
    let bullets: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "item")
        .map(|record| record.field("bullet"))
        .collect();
    assert_eq!(
        bullets,
        [Some("-"), Some("-"), Some("-"), Some("1."), Some("2)")]
    );
}

#[test]
fn org_scheme_context_list_boundaries_keep_headlines_and_marker_types_distinct() {
    let source = "* H\n  * item\n\n\nnext\n1. a\n- b\n\t- c\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("mixed list boundaries remain lossless");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("mixed marker types project Elements");
    for (kind, expected) in [("headline", 1), ("plain-list", 4), ("item", 4)] {
        assert_eq!(
            records.iter().filter(|record| record.kind == kind).count(),
            expected
        );
    }
}

#[test]
fn org_scheme_context_algorithm_aot_projects_inline_link_objects() {
    let source = "go [[https://a][α]] and [[id:b]]\n[[broken\n- [[file:x][item]]\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme inline links build a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme inline links project Org Objects");
    let links: Vec<_> = records
        .iter()
        .filter(|record| record.kind == "link")
        .map(|record| (record.field("path"), record.field("description")))
        .collect();
    assert_eq!(
        links,
        [
            (Some("https://a"), Some("α")),
            (Some("id:b"), None),
            (Some("file:x"), Some("item"))
        ]
    );
}

#[test]
fn org_scheme_context_algorithm_aot_projects_recursive_containers() {
    let source = "#+begin_quote\ntext\n- item\n#+end_quote\n#+BEGIN: note\nbody\n#+END:\n:LOGBOOK:\nentry\n:END:\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("Scheme recursive containers build a lossless Rowan tree");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("Scheme containers project Org Elements");
    let quote = records
        .iter()
        .find(|record| record.kind == "quote-block")
        .expect("quote block Element");
    let list = records
        .iter()
        .find(|record| record.kind == "plain-list")
        .expect("nested plain list Element");
    assert_eq!(list.parent_id, Some(quote.id));
    let dynamic = records
        .iter()
        .find(|record| record.kind == "dynamic-block")
        .expect("dynamic block Element");
    assert_eq!(dynamic.field("name"), Some("note"));
    let drawer = records
        .iter()
        .find(|record| record.kind == "drawer")
        .expect("named drawer Element");
    assert_eq!(drawer.field("name"), Some("LOGBOOK"));
}

#[test]
fn org_scheme_context_algorithm_aot_projects_indented_properties() {
    let source = "* H\n  :PROPERTIES:\n  :ID: x\n  :END:\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("indented property drawer remains lossless");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("indented property projects as an Element");
    let property = records
        .iter()
        .find(|record| record.kind == "node-property")
        .expect("node property Element");
    assert_eq!(property.field("key"), Some("ID"));
    assert_eq!(property.field("value"), Some("x"));
}

#[test]
fn org_scheme_context_algorithm_aot_keeps_nonidentifier_property_keys() {
    let source = "* H\n:PROPERTIES:\n:A+B: yes\n:END:\n";
    let events = generated_context_events::parse_org_rowan_events(source);
    let parsed = parse_generated_events(
        org_language_spec(),
        generated_context_events::PARSER_DIGEST,
        source,
        &events,
    )
    .expect("nonidentifier property key remains lossless");
    assert_eq!(parsed.syntax().to_string(), source);
    let records = project_syntax_graph(org_language_spec(), org_graph_spec(), &parsed.syntax())
        .expect("source-backed property projects as an Element");
    let property = records
        .iter()
        .find(|record| record.kind == "node-property")
        .expect("node property Element");
    assert_eq!(property.field("key"), Some("A+B"));
    assert_eq!(property.field("value"), Some("yes"));
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
