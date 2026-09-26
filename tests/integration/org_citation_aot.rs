//! Citation Objects are Scheme-owned and source-backed in Rowan.

#[test]
fn scheme_declared_citations_project_into_rowan_and_graph() {
    check_org_aot_element!("[cite:@doe2020]\n", "citation-reference", "key" => "doe2020");
    check_org_aot_element!(
        "[cite:@key\n[cite:@next]\n",
        "citation-reference",
        "key" => "next"
    );
    for invalid in ["[cite:no key]\n", "[cite/:@key]\n"] {
        let document = orgize::org_aot::parse_org_aot(invalid)
            .expect("invalid citation remains lossless text");
        assert!(
            !document
                .records()
                .iter()
                .any(|record| record.kind == "citation")
        );
        assert_eq!(document.syntax().to_string(), invalid);
    }
}

#[test]
fn citation_references_keep_distinct_keys_and_source_fields() {
    let source = "[cite/text:see @doe2020 p. 42; cf. @roe2021]\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("Scheme-owned references compile through Rowan");
    let references: Vec<_> = document
        .records()
        .iter()
        .filter(|record| record.kind == "citation-reference")
        .collect();
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].field("prefix"), Some("see "));
    assert_eq!(references[0].field("key"), Some("doe2020"));
    assert_eq!(references[0].field("suffix"), Some(" p. 42"));
    assert_eq!(references[1].field("prefix"), Some(" cf. "));
    assert_eq!(references[1].field("key"), Some("roe2021"));
    assert_eq!(document.syntax().to_string(), source);
}
