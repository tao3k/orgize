//! The Org-owned Scheme grammar must compile to a real Rowan parser product.

#[path = "../../languages/org/v1/generated/parser.rs"]
mod grammar;
#[path = "../../languages/org/v1/generated/scanner.rs"]
mod scanner;

fn parse(source: &str) -> gerbil_parser_rowan::Parse {
    let tokens = scanner::scan(source);
    gerbil_parser_rowan::parse_scanned(&grammar::LANGUAGE, source, scanner::SCANNER_DIGEST, &tokens)
        .unwrap_or_else(|error| panic!("AOT Org parser rejected source: {error:?}"))
}

#[test]
fn scheme_grammar_scanner_and_rowan_engine_form_one_lossless_path() {
    let source = "* Real\r\n#+BEGIN_SRC rust\r\n* code, not a headline\r\n#+END_SRC\r\né\n";
    let parsed = parse(source);
    let root = parsed.syntax();
    assert_eq!(root.to_string(), source);
    assert_eq!(parsed.kind_name(root.kind()), Some("OrgFile"));

    let kinds: Vec<_> = root
        .descendants()
        .filter_map(|node| parsed.kind_name(node.kind()))
        .collect();
    assert!(kinds.contains(&"OrgHeadline"), "{kinds:?}");
    assert!(kinds.contains(&"OrgSourceBlock"), "{kinds:?}");
    assert_eq!(
        kinds.iter().filter(|kind| **kind == "OrgHeadline").count(),
        1,
        "headlines inside source blocks must remain body text"
    );
    assert_eq!(
        parsed.receipt().grammar_digest,
        grammar::LANGUAGE.grammar_digest
    );
    assert_eq!(
        parsed.receipt().scanner_digest,
        Some(scanner::SCANNER_DIGEST)
    );
}

#[test]
fn malformed_source_block_is_not_silently_accepted_as_text() {
    let source = "#+begin_src rust\nmissing end\n";
    let tokens = scanner::scan(source);
    let error = gerbil_parser_rowan::parse_scanned(
        &grammar::LANGUAGE,
        source,
        scanner::SCANNER_DIGEST,
        &tokens,
    )
    .expect_err("missing block end must not become a successful all-text tree");
    assert_ne!(error.diagnostic.reason_kind, "invalid-aot-artifact");
}
