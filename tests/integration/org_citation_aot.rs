//! Citation Objects are Scheme-owned and source-backed in Rowan.

#[test]
fn scheme_declared_citations_project_into_rowan_and_graph() {
    check_org_aot_element!("[cite:@doe2020]\n", "citation", "body" => "@doe2020");
    check_org_aot_element!(
        "[cite/text:see @doe2020 p. 42; cf. @roe2021]\n",
        "citation",
        "body" => "see @doe2020 p. 42; cf. @roe2021"
    );
    check_org_aot_element!(
        "[cite:@key\n[cite:@next]\n",
        "citation",
        "body" => "@next"
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
