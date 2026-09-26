//! Scheme-owned footnote references and definitions in the Rowan Element graph.

macro_rules! check_org_footnote {
    ($record:expr, $source:expr, $kind:expr, $label:expr, $definition:expr, $literal:expr) => {{
        let record = $record;
        let range = record.range;
        assert_eq!(record.kind, $kind);
        assert_eq!(record.field("label"), $label);
        assert_eq!(record.field("definition"), $definition);
        assert_eq!(
            &$source[usize::from(range.start())..usize::from(range.end())],
            $literal
        );
    }};
}

#[test]
fn scheme_footnotes_connect_source_backed_reference_and_definition() {
    let source = "Text[fn:note] and [fn::a [b]]\n[fn:note] The answer.\n* Next\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("Scheme footnotes parse");
    assert_eq!(document.syntax().to_string(), source);
    let records = document.records();
    let references = records
        .iter()
        .filter(|record| record.kind == "footnote-reference")
        .collect::<Vec<_>>();
    let definitions = records
        .iter()
        .filter(|record| record.kind == "footnote-definition")
        .collect::<Vec<_>>();
    assert_eq!(references.len(), 2);
    assert_eq!(definitions.len(), 1);
    check_org_footnote!(
        references[0],
        source,
        "footnote-reference",
        Some("note"),
        None,
        "[fn:note]"
    );
    check_org_footnote!(
        references[1],
        source,
        "footnote-reference",
        None,
        Some("a [b]"),
        "[fn::a [b]]"
    );
    check_org_footnote!(
        definitions[0],
        source,
        "footnote-definition",
        Some("note"),
        None,
        "[fn:note] The answer.\n"
    );
}

#[test]
fn scheme_footnote_definition_ends_before_two_blank_lines() {
    let source = "[fn:a] first\n\ncontinued\n\n\noutside\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("multiline footnote parses");
    assert_eq!(document.syntax().to_string(), source);
    let definitions = document
        .records()
        .iter()
        .filter(|record| record.kind == "footnote-definition")
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 1);
    check_org_footnote!(
        definitions[0],
        source,
        "footnote-definition",
        Some("a"),
        None,
        "[fn:a] first\n\ncontinued\n"
    );
}

#[test]
fn scheme_footnote_definition_keeps_one_trailing_blank_at_eof() {
    let source = "[fn:a] first\n\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("footnote EOF parses");
    assert_eq!(document.syntax().to_string(), source);
    let definitions = document
        .records()
        .iter()
        .filter(|record| record.kind == "footnote-definition")
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 1);
    check_org_footnote!(
        definitions[0],
        source,
        "footnote-definition",
        Some("a"),
        None,
        source
    );
}

#[test]
fn scheme_footnote_definition_keeps_one_blank_before_the_next_definition() {
    let source = "[fn:a] one\n\n[fn:b] two\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("adjacent footnotes parse");
    assert_eq!(document.syntax().to_string(), source);
    let definitions = document
        .records()
        .iter()
        .filter(|record| record.kind == "footnote-definition")
        .collect::<Vec<_>>();
    assert_eq!(definitions.len(), 2);
    check_org_footnote!(
        definitions[0],
        source,
        "footnote-definition",
        Some("a"),
        None,
        "[fn:a] one\n\n"
    );
    check_org_footnote!(
        definitions[1],
        source,
        "footnote-definition",
        Some("b"),
        None,
        "[fn:b] two\n"
    );
}

#[test]
fn scheme_footnotes_reject_invalid_labels_and_opaque_source_text() {
    let source = "[fn:] invalid\n[fn:bad name]\n#+begin_src text\n[fn:hidden]\n#+end_src\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("invalid footnotes recover");
    assert_eq!(document.syntax().to_string(), source);
    assert_eq!(
        document
            .records()
            .iter()
            .filter(|record| matches!(record.kind, "footnote-reference" | "footnote-definition"))
            .count(),
        0
    );
}
