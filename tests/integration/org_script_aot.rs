//! Scheme-owned script Objects must survive AOT into Rowan and Element Query.

#[test]
fn scheme_script_objects_project_with_source_backed_values() {
    check_org_aot_element!("x_abc\n", "subscript", "value" => "abc");
    check_org_aot_element!("x_{a{b}c}\n", "subscript", "value" => "a{b}c");
    check_org_aot_element!("x^2\n", "superscript", "value" => "2");
    check_org_aot_element!("x^*\n", "superscript", "value" => "*");
}

#[test]
fn script_guards_do_not_consume_identifiers_or_markup() {
    let source = "AB_2O x^a, _hello_\n";
    let document = orgize::org_aot::parse_org_aot(source)
        .expect("invalid scripts and underline remain lossless");
    assert!(
        document
            .records()
            .iter()
            .all(|record| record.kind != "subscript" && record.kind != "superscript")
    );
    assert!(
        document
            .records()
            .iter()
            .any(|record| record.kind == "underline")
    );
    assert_eq!(document.syntax().to_string(), source);
}
