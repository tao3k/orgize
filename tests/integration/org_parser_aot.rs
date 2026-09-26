//! Scheme POO declaration -> gerbil-parser AOT table -> contextual Rowan CST.

use gerbil_parser_rowan::SyntaxNode;

fn parse(source: &str) -> SyntaxNode {
    let parsed = orgize::org_aot::parse_org_aot(source)
        .unwrap_or_else(|error| panic!("Org event AOT rejected source: {error:?}"));
    assert_eq!(parsed.receipt().language, "org");
    assert_eq!(
        parsed.receipt().grammar_digest,
        orgize::org_aot::org_language_spec().grammar_digest
    );
    assert_eq!(
        parsed.receipt().parser_digest,
        Some(orgize::org_aot::org_event_parser_digest())
    );
    parsed.syntax()
}

fn name(node: &SyntaxNode) -> &'static str {
    orgize::org_aot::org_language_spec().kinds[usize::from(node.kind().0)].name
}

fn token_name(token: &gerbil_parser_rowan::SyntaxToken) -> &'static str {
    orgize::org_aot::org_language_spec().kinds[usize::from(token.kind().0)].name
}

macro_rules! check_org_aot_element {
    ($source:expr, $kind:expr, $field:expr => $value:expr) => {{
        let document = orgize::org_aot::parse_org_aot($source)
            .expect("Scheme-owned Org Element parser accepts the source");
        let elements: Vec<_> = document
            .records()
            .iter()
            .filter(|record| record.kind == $kind)
            .collect();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].field($field), Some($value));
        assert_eq!(document.syntax().to_string(), $source);
    }};
}

#[test]
fn scheme_declared_macro_objects_project_into_rowan_and_graph() {
    check_org_aot_element!("{{{issue(42)}}}\n", "macro", "name" => "issue");
    let document =
        orgize::org_aot::parse_org_aot("{{{issue(42)}}}\n").expect("Scheme-owned Org macro object");
    let macro_record = document
        .records()
        .iter()
        .find(|record| record.kind == "macro")
        .expect("macro graph record");
    assert_eq!(macro_record.field("arguments"), Some("42"));
    assert_eq!(document.syntax().to_string(), "{{{issue(42)}}}\n");
    let malformed = orgize::org_aot::parse_org_aot("{{{9bad}}} {{{broken\n")
        .expect("invalid macros are lossless text");
    assert!(
        !malformed
            .records()
            .iter()
            .any(|record| record.kind == "macro")
    );
    assert_eq!(malformed.syntax().to_string(), "{{{9bad}}} {{{broken\n");
}

#[test]
fn scheme_declared_entities_require_catalog_names_and_preserve_postfix() {
    check_org_aot_element!("\\alpha{}\n", "entity", "name" => "alpha");
    let document =
        orgize::org_aot::parse_org_aot("\\alpha{} \\_   \n").expect("Scheme-owned Org entities");
    let entities: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "entity")
        .collect();
    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0].field("post"), Some("{}"));
    assert_eq!(entities[1].field("name"), Some("_"));
    assert_eq!(entities[1].field("post"), Some("   "));
    assert_eq!(document.syntax().to_string(), "\\alpha{} \\_   \n");
    let unknown = orgize::org_aot::parse_org_aot("\\unknown \\centaur\n")
        .expect("unknown entity names are text");
    assert!(
        !unknown
            .records()
            .iter()
            .any(|record| record.kind == "entity")
    );
    let adjacent = orgize::org_aot::parse_org_aot("\\alpha\\beta [[https://example.org]]\n")
        .expect("entity boundaries keep the next Object visible");
    assert_eq!(
        adjacent
            .records()
            .iter()
            .filter(|record| record.kind == "entity")
            .count(),
        2
    );
    assert!(
        adjacent
            .records()
            .iter()
            .any(|record| record.kind == "link")
    );
}

#[test]
fn scheme_declared_babel_call_is_not_a_generic_keyword() {
    let source = "#+CALL: build(input=42)\n";
    check_org_aot_element!(source, "babel-call", "value" => "build(input=42)");
    let document = orgize::org_aot::parse_org_aot(source).expect("Babel Call element");
    assert!(
        !document
            .records()
            .iter()
            .any(|record| record.kind == "keyword")
    );
}

#[test]
fn scheme_emphasis_objects_project_into_rowan_and_element_graph() {
    let source = "*bold* /italic/ _under_ +strike+\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-owned emphasis Objects parse through the event AOT");
    for (kind, value) in [
        ("bold", "bold"),
        ("italic", "italic"),
        ("underline", "under"),
        ("strike-through", "strike"),
    ] {
        let records: Vec<_> = document
            .records()
            .iter()
            .filter(|record| record.kind == kind)
            .collect();
        assert_eq!(records.len(), 1, "{kind}");
        assert_eq!(records[0].field("value"), Some(value), "{kind}");
    }
    assert_eq!(document.syntax().to_string(), source);
}

#[test]
fn headings_form_nested_sections_and_closed_blocks_remain_lossless() {
    let source = "é\r\n* Parent\n#+BEGIN_SRC rust\ncode\n#+END_SRC\n** Child\nbody\r* Sibling\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(name(&root), "OrgFile");
    let sections: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgSection")
        .collect();
    assert_eq!(sections.len(), 3);
    assert_eq!(name(&sections[0].parent().unwrap()), "OrgFile");
    assert_eq!(name(&sections[1].parent().unwrap()), "OrgSection");
    assert_eq!(name(&sections[2].parent().unwrap()), "OrgFile");
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        3
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
    );
}

#[test]
fn scheme_declared_paragraphs_preserve_line_breaks_and_link_ancestry() {
    let source = "* Task\r\nalpha\r\nbeta\n\n[[https://example.test][inside]]\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    let paragraphs: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgParagraph")
        .collect();
    assert_eq!(paragraphs.len(), 2);
    assert_eq!(paragraphs[0].to_string(), "alpha\r\nbeta\n\n");
    assert_eq!(
        paragraphs[1].to_string(),
        "[[https://example.test][inside]]\n"
    );
    assert_eq!(name(&paragraphs[0].parent().unwrap()), "OrgSection");
    let link = root
        .descendants()
        .find(|node| name(node) == "OrgLink")
        .unwrap();
    assert_eq!(name(&link.parent().unwrap()), "OrgTextLine");
    assert_eq!(
        name(&link.parent().unwrap().parent().unwrap()),
        "OrgParagraph"
    );
}

#[test]
fn scheme_declared_table_builds_rows_and_cells_without_paragraph_claims() {
    let source = "* Data\n| Name | Value |\n|------+-------|\n| é\\|x | 42 |\nAfter\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    let tables: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgTable")
        .collect();
    assert_eq!(tables.len(), 1);
    assert_eq!(name(&tables[0].parent().unwrap()), "OrgSection");
    assert_eq!(
        tables[0]
            .descendants()
            .filter(|node| name(node) == "OrgTableRow")
            .count(),
        2
    );
    assert_eq!(
        tables[0]
            .descendants()
            .filter(|node| name(node) == "OrgTableRuleRow")
            .count(),
        1
    );
    let cells: Vec<_> = tables[0]
        .descendants()
        .filter(|node| name(node) == "OrgTableCell")
        .collect();
    assert_eq!(cells.len(), 4);
    assert_eq!(cells[2].to_string(), " é\\|x ");
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgParagraph")
            .count(),
        1
    );
}

#[test]
fn scheme_declared_greater_blocks_keep_distinct_element_kinds_and_export_backend() {
    let source = "* Blocks\n#+begin_quote\nquoted\n#+end_quote\n#+begin_example\nliteral\n#+end_example\n#+begin_verse\nverse\n#+end_verse\n#+begin_center\ncentered\n#+end_center\n#+begin_comment\nhidden\n#+end_comment\n#+begin_export html\n<b>raw</b>\n#+end_export\nAfter\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    let expected = [
        ("OrgQuoteBlock", "quote-block"),
        ("OrgExampleBlock", "example-block"),
        ("OrgVerseBlock", "verse-block"),
        ("OrgCenterBlock", "center-block"),
        ("OrgCommentBlock", "comment-block"),
        ("OrgExportBlock", "export-block"),
    ];
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("greater blocks use the production Scheme-owned projection");
    let records = document.records();
    for (syntax_kind, graph_kind) in expected {
        let node = root
            .descendants()
            .find(|node| name(node) == syntax_kind)
            .unwrap_or_else(|| panic!("missing {syntax_kind}"));
        assert_eq!(name(&node.parent().unwrap()), "OrgSection");
        assert!(records.iter().any(|record| record.kind == graph_kind));
    }
    let export = records
        .iter()
        .find(|record| record.kind == "export-block")
        .expect("export block is a typed Element");
    assert_eq!(export.field("backend"), Some("html"));
    assert_eq!(export.field("body"), Some("<b>raw</b>\n"));
    assert_eq!(
        records
            .iter()
            .find(|record| record.kind == "example-block")
            .and_then(|record| record.field("body")),
        Some("literal\n")
    );
    assert_eq!(
        records
            .iter()
            .find(|record| record.kind == "comment-block")
            .and_then(|record| record.field("body")),
        Some("hidden\n")
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgParagraph")
            .count(),
        4
    );
}

#[test]
fn unmatched_greater_block_does_not_swallow_following_headline() {
    let source = "* First\n#+begin_quote\nunclosed\n** Next\nvisible\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgQuoteBlock")
            .count(),
        0
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSection")
            .count(),
        2
    );
}

#[test]
fn recursive_greater_blocks_project_inner_elements_but_literal_blocks_do_not() {
    let source = "* Task\n#+begin_quote\n[[id:inside]]\n| a | b |\n#+begin_example\n[[id:literal]]\n#+end_example\n#+end_quote\n#+begin_verse\n[[id:verse]]\n#+end_verse\n#+begin_center\n[[id:center]]\n#+end_center\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    let quote = root
        .descendants()
        .find(|node| name(node) == "OrgQuoteBlock")
        .expect("quote block is present");
    assert_eq!(
        quote
            .descendants()
            .filter(|node| name(node) == "OrgLink")
            .count(),
        1
    );
    assert_eq!(
        quote
            .descendants()
            .filter(|node| name(node) == "OrgTable")
            .count(),
        1
    );
    assert_eq!(
        quote
            .descendants()
            .filter(|node| name(node) == "OrgExampleBlock")
            .count(),
        1
    );
    let records = gerbil_parser_rowan::project_syntax_graph(
        orgize::org_aot::org_language_spec(),
        orgize::org_aot::org_graph_spec(),
        &root,
    )
    .expect("nested Element graph follows recursive block ancestry");
    let quote_record = records
        .iter()
        .find(|record| record.kind == "quote-block")
        .expect("quote is projected");
    assert!(records.iter().any(|record| {
        record.kind == "link"
            && record.field("path") == Some("id:inside")
            && record
                .parent_id
                .is_some_and(|parent| records[parent].parent_id == Some(quote_record.id))
    }));
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "link")
            .count(),
        3
    );
}

#[test]
fn unclosed_source_block_recovers_as_text_before_the_next_headline() {
    let source = "* Open\r\n#+begin_src rust\r\n** source text\r\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        2
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        0
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgTextLine")
            .count(),
        1
    );
}

#[test]
fn closed_source_block_masks_heading_looking_body_lines() {
    let source = "* One\n#+begin_src rust\n** Next\n#+end_src\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        1
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
    );
}

#[test]
fn many_sibling_sections_keep_exact_source_order() {
    let source = "* item\n".repeat(10_000);
    let root = parse(&source);
    assert_eq!(root.to_string(), source);
    assert_eq!(root.children().count(), 10_000);
}

#[test]
fn git_tracked_org_document_is_lossless_at_the_current_structural_boundary() {
    let source = include_str!("../fixtures/org-elements/representative.org");
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSection")
            .count(),
        4
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
    );
}

#[test]
fn git_tracked_list_items_have_scheme_aot_ancestry_and_typed_bullets() {
    let source = include_str!("../fixtures/org-elements/representative.org");
    let root = parse(source);
    let lists: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgPlainList")
        .collect();
    assert_eq!(lists.len(), 1);
    let items: Vec<_> = lists[0]
        .children()
        .filter(|node| name(node) == "OrgListItem")
        .collect();
    assert_eq!(items.len(), 2);
    for item in &items {
        let bullet = item
            .children_with_tokens()
            .filter_map(rowan::NodeOrToken::into_token)
            .find(|token| token_name(token) == "ListBullet")
            .expect("every admitted item has a typed bullet");
        assert_eq!(bullet.text(), "-");
        let range = item.text_range();
        assert_eq!(
            &source[usize::from(range.start())..usize::from(range.end())],
            item.to_string()
        );
    }
    let records = gerbil_parser_rowan::project_syntax_graph(
        orgize::org_aot::org_language_spec(),
        orgize::org_aot::org_graph_spec(),
        &root,
    )
    .expect("list projection uses Scheme-owned Element kinds");
    let list = records
        .iter()
        .find(|record| record.kind == "plain-list")
        .expect("tracked fixture projects one list");
    let children: Vec<_> = records
        .iter()
        .filter(|record| record.parent_id == Some(list.id) && record.kind == "item")
        .collect();
    assert_eq!(children.len(), 2);
    assert!(
        children
            .iter()
            .all(|item| item.field("bullet") == Some("-"))
    );
}

#[test]
fn keyed_lines_obey_heading_context_and_project_keyword_fields() {
    let source = "#+TITLE: α fixture\n* Task\nSCHEDULED: <2026-09-24 Thu> DEADLINE: <2026-09-25 Fri>\nBody\nSCHEDULED: ordinary prose\n#+begin_src text\nliteral\n#+end_src\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    let planning: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgPlanning")
        .collect();
    assert_eq!(planning.len(), 1);
    assert_eq!(name(&planning[0].parent().unwrap()), "OrgSection");
    let keys: Vec<_> = planning[0]
        .children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(|token| token_name(token) == "PlanningKey")
        .map(|token| token.text().to_string())
        .collect();
    assert_eq!(keys, ["SCHEDULED", "DEADLINE"]);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgKeyword")
            .count(),
        1
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .count(),
        1
    );
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("keywords and planning use the production Scheme-owned projection");
    let records = document.records();
    let keyword = records
        .iter()
        .find(|record| record.kind == "keyword")
        .expect("one keyword");
    assert_eq!(keyword.field("key"), Some("TITLE"));
    assert_eq!(keyword.field("value"), Some("α fixture"));
    let planning = records
        .iter()
        .find(|record| record.kind == "planning")
        .expect("one planning element");
    assert_eq!(
        planning.values("key").collect::<Vec<_>>(),
        ["SCHEDULED", "DEADLINE"]
    );
    assert_eq!(
        planning.values("value").collect::<Vec<_>>(),
        ["<2026-09-24 Thu>", "<2026-09-25 Fri>"]
    );
}

#[test]
fn clock_is_a_source_backed_element_between_paragraphs() {
    let source = "* Task\nBefore\nCLOCK: [2026-09-24 Thu 10:00]--[2026-09-24 Thu 11:00] => 1:00\nAfter\n#+begin_example\nCLOCK: literal\n#+end_example\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-owned clock rule admits a standalone element");
    let root = document.syntax();
    assert_eq!(root.to_string(), source);
    let clocks: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgClock")
        .collect();
    assert_eq!(clocks.len(), 1);
    let clock = &clocks[0];
    assert_eq!(name(&clock.parent().unwrap()), "OrgSection");
    let range = clock.text_range();
    assert_eq!(
        &source[usize::from(range.start())..usize::from(range.end())],
        clock.to_string()
    );
    let paragraphs: Vec<_> = root
        .descendants()
        .filter(|node| name(node) == "OrgParagraph")
        .collect();
    assert_eq!(paragraphs.len(), 2);
    assert!(
        paragraphs
            .iter()
            .all(|paragraph| !paragraph.to_string().contains("CLOCK:"))
    );
    let records = document.records();
    let clock = records
        .iter()
        .find(|record| record.kind == "clock")
        .expect("clock Element is projected into the query graph");
    assert_eq!(
        clock.field("value"),
        Some("[2026-09-24 Thu 10:00]--[2026-09-24 Thu 11:00] => 1:00")
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == "clock")
            .count(),
        1
    );
}

#[test]
fn tracked_fixture_has_scheme_owned_keywords_and_planning() {
    let source = include_str!("../fixtures/org-elements/representative.org");
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgKeyword")
            .count(),
        4
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgPlanning")
            .count(),
        1
    );
}

#[test]
fn contract_scope_mvp_inputs_expose_drawers_and_node_properties() {
    let fixtures = [
        (
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/contracts.org"
            ),
            8,
            20,
            4,
            0,
            0,
        ),
        (
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
            ),
            4,
            6,
            0,
            6,
            3,
        ),
    ];
    for (source, drawers, properties, source_blocks, contract_org_properties, links) in fixtures {
        let root = parse(source);
        assert_eq!(root.to_string(), source);
        let headlines: Vec<_> = root
            .descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .collect();
        assert_eq!(headlines.len(), 8);
        for headline in &headlines {
            let title = headline
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .find(|token| token_name(token) == "HeadlineTitle")
                .expect("fixture headline has a typed title");
            let range = title.text_range();
            assert_eq!(
                &source[usize::from(range.start())..usize::from(range.end())],
                title.text()
            );
            assert!(!title.text().is_empty());
        }
        assert_eq!(
            root.descendants()
                .filter(|node| name(node) == "OrgPropertyDrawer")
                .count(),
            drawers
        );
        assert_eq!(
            root.descendants()
                .filter(|node| name(node) == "OrgSourceBlock")
                .count(),
            source_blocks
        );
        let languages: Vec<_> = root
            .descendants()
            .filter(|node| name(node) == "OrgSourceBlock")
            .map(|block| {
                let token = block
                    .children_with_tokens()
                    .filter_map(rowan::NodeOrToken::into_token)
                    .find(|token| token_name(token) == "SourceLanguage")
                    .expect("every fixture source block declares its language");
                let range = token.text_range();
                assert_eq!(
                    &source[usize::from(range.start())..usize::from(range.end())],
                    token.text()
                );
                token.text().to_owned()
            })
            .collect();
        assert_eq!(languages, vec!["org-contract"; source_blocks]);
        let link_nodes: Vec<_> = root
            .descendants()
            .filter(|node| name(node) == "OrgLink")
            .collect();
        assert_eq!(link_nodes.len(), links);
        for link in link_nodes {
            let range = link.text_range();
            assert_eq!(
                &source[usize::from(range.start())..usize::from(range.end())],
                link.to_string()
            );
            let tokens: Vec<_> = link
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .collect();
            assert_eq!(
                tokens
                    .iter()
                    .find(|token| token_name(token) == "LinkTarget")
                    .expect("link has a typed target")
                    .text(),
                "https://example.test"
            );
            assert!(
                tokens
                    .iter()
                    .any(|token| token_name(token) == "LinkDescription")
            );
        }
        let property_nodes: Vec<_> = root
            .descendants()
            .filter(|node| name(node) == "OrgNodeProperty")
            .collect();
        assert_eq!(property_nodes.len(), properties);
        let mut contract_org_count = 0;
        for node in property_nodes {
            let range = node.text_range();
            let original = &source[usize::from(range.start())..usize::from(range.end())];
            assert_eq!(node.to_string(), original);
            assert!(original.trim_start().starts_with(':'));
            let tokens: Vec<_> = node
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .collect();
            let key = tokens
                .iter()
                .find(|token| token_name(token) == "PropertyKey")
                .expect("every admitted node-property has a typed key");
            let value = tokens
                .iter()
                .find(|token| token_name(token) == "PropertyValue")
                .expect("fixture node-properties have typed values");
            for token in [key, value] {
                let range = token.text_range();
                assert_eq!(
                    &source[usize::from(range.start())..usize::from(range.end())],
                    token.text()
                );
            }
            contract_org_count += usize::from(key.text() == "CONTRACT_ORG");
        }
        assert_eq!(contract_org_count, contract_org_properties);
    }
}

#[test]
fn invalid_property_drawer_recovers_without_claiming_node_properties() {
    let source = "* Task\n:PROPERTIES:\nnot a property\n:END:\n** Next\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgPropertyDrawer")
            .count(),
        0
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        2
    );
}

#[test]
fn contract_scope_graph_projection_uses_only_scheme_owned_cst_rules() {
    let fixtures = [
        (
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/contracts.org"
            ),
            41,
            4,
            0,
            0,
        ),
        (
            include_str!(
                "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
            ),
            25,
            0,
            3,
            3,
        ),
    ];
    for (source, expected_records, expected_blocks, expected_links, expected_paragraphs) in fixtures
    {
        let root = parse(source);
        let records = gerbil_parser_rowan::project_syntax_graph(
            orgize::org_aot::org_language_spec(),
            orgize::org_aot::org_graph_spec(),
            &root,
        )
        .expect("Scheme AOT graph rule must match the same grammar");
        assert_eq!(records.len(), expected_records);
        assert_eq!(records[0].kind, "org-data");
        assert_eq!(records[0].parent_id, None);
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == "headline")
                .count(),
            8
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == "src-block")
                .count(),
            expected_blocks
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == "link")
                .count(),
            expected_links
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == "paragraph")
                .count(),
            expected_paragraphs
        );
        for record in &records {
            assert_eq!(records[record.id].id, record.id);
            if let Some(parent_id) = record.parent_id {
                assert!(records[parent_id].child_ids.contains(&record.id));
            }
            let range = record.range;
            assert_eq!(
                &source[usize::from(range.start())..usize::from(range.end())],
                root.descendants()
                    .find(|node| node.text_range() == range)
                    .expect("graph range belongs to a Rowan node")
                    .to_string()
            );
        }
        for record in records.iter().filter(|record| record.kind == "link") {
            assert_eq!(record.field("path"), Some("https://example.test"));
            let parent = &records[record.parent_id.expect("link has a paragraph parent")];
            assert_eq!(parent.kind, "paragraph");
            let section = &records[parent.parent_id.expect("paragraph has a section parent")];
            assert_eq!(section.kind, "headline");
            assert_eq!(section.field("title"), Some("Evidence"));
        }
        for record in records.iter().filter(|record| record.kind == "src-block") {
            assert_eq!(record.field("language"), Some("org-contract"));
            assert!(
                record
                    .field("body")
                    .is_some_and(|body| body.contains("(assert"))
            );
        }
    }
}

#[test]
fn scheme_aot_contract_evaluates_generated_org_element_ancestry() {
    let source = include_str!(
        "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org"
    );
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("production Org AOT entrypoint parses and projects Elements");
    assert_eq!(document.syntax().to_string(), source);
    assert_eq!(
        document.receipt().grammar_digest,
        orgize::org_aot::org_language_spec().grammar_digest
    );
    let records = document.records();
    assert_eq!(orgize::org_aot::org_contract_pack().rules.len(), 4);
    let evidence = records
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("Evidence"))
        .expect("fixture has an Evidence headline");
    let scope = evidence.parent_id.expect("Evidence has a parent headline");
    let contract = orgize::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "section.scope.v1")
        .expect("Scheme AOT pack includes the subtree contract");
    let results = document
        .evaluate_contract(
            contract,
            orgize::contract_feature::ContractScopeNodeId(scope),
        )
        .expect("Scheme-AOT contract has valid Element bindings");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].assertion_id, "section.has-evidence-link");
    assert_eq!(results[0].matched_count, 1);
    assert!(results[0].passed);

    let document_contract = orgize::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "document.headlines.v1")
        .expect("Scheme AOT pack includes the document contract");
    let document_results = document
        .evaluate_contract(
            document_contract,
            orgize::contract_feature::ContractScopeNodeId(0),
        )
        .expect("document contract uses the same Element graph");
    assert_eq!(document_results.len(), 1);
    assert!(document_results[0].passed);
    assert!(document_results[0].matched_count >= 2);
    assert_eq!(
        document.evaluate_contract(
            document_contract,
            orgize::contract_feature::ContractScopeNodeId(scope),
        ),
        Err(orgize::contract_feature::ContractExecutionError::InvalidScope)
    );

    let property_contract = orgize::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "document.properties.v1")
        .expect("Scheme AOT pack includes the document property contract");
    let property_results = document
        .evaluate_contract(
            property_contract,
            orgize::contract_feature::ContractScopeNodeId(0),
        )
        .expect("document property query is admitted");
    assert_eq!(property_results.len(), 1);
    assert!(property_results[0].passed);

    let override_scope = records
        .iter()
        .find(|record| {
            record.kind == "headline" && record.field("title") == Some("Override Parent")
        })
        .expect("fixture has the override section")
        .id;
    let override_contract = orgize::org_aot::org_contract_pack()
        .rules
        .iter()
        .find(|rule| rule.id == "section.override-title.v1")
        .expect("Scheme AOT pack includes the override title contract");
    let override_results = document
        .evaluate_contract(
            override_contract,
            orgize::contract_feature::ContractScopeNodeId(override_scope),
        )
        .expect("scope-bound title containment is admitted");
    assert_eq!(override_results.len(), 1);
    assert_eq!(override_results[0].matched_count, 1);
    assert!(override_results[0].passed);

    let stale = orgize::contract_feature::ContractRule {
        graph_digest: "outdated-projection",
        ..*contract
    };
    assert_eq!(
        document.evaluate_contract(&stale, orgize::contract_feature::ContractScopeNodeId(scope),),
        Err(orgize::contract_feature::ContractExecutionError::StaleGraph)
    );
}

#[test]
fn incomplete_link_remains_lossless_text() {
    let source = "* One\nparagraph [[unfinished\n** Next\n";
    let root = parse(source);
    assert_eq!(root.to_string(), source);
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgLink")
            .count(),
        0
    );
    assert_eq!(
        root.descendants()
            .filter(|node| name(node) == "OrgHeadline")
            .count(),
        2
    );
}
