//! Scheme-owned script Objects must survive AOT into Rowan and Element Query.

use orgize::ParseConfig;
use orgize::config::UseSubSuperscript;
use orgize::org_aot::{parse_org_aot, parse_org_aot_with_config};

fn configured_inline_script_policy_reaches_scheme_aot_helper() {
    let source = "x_abc y_{z}\n";
    let count = |policy| {
        let config = ParseConfig {
            use_sub_superscript: policy,
            ..ParseConfig::default()
        };
        let document = parse_org_aot_with_config(source, &config).expect("configured AOT parses");
        assert_eq!(document.syntax().to_string(), source);
        document
            .records()
            .iter()
            .filter(|record| record.kind == "subscript")
            .count()
    };
    assert_eq!(
        parse_org_aot(source)
            .expect("default AOT parses")
            .records()
            .iter()
            .filter(|record| record.kind == "subscript")
            .count(),
        2
    );
    assert_eq!(count(UseSubSuperscript::Nil), 0);
    assert_eq!(count(UseSubSuperscript::Brace), 1);
    assert_eq!(count(UseSubSuperscript::True), 2);

    let list_config = ParseConfig {
        use_sub_superscript: UseSubSuperscript::Nil,
        ..ParseConfig::default()
    };
    let list_source = "- x_abc\n";
    let list = parse_org_aot_with_config(list_source, &list_config)
        .expect("list paragraph uses configured helper");
    assert_eq!(list.syntax().to_string(), list_source);
    assert!(
        list.records()
            .iter()
            .all(|record| record.kind != "subscript")
    );
}

fn scheme_script_objects_project_with_source_backed_values() {
    check_org_aot_element!("x_abc\n", "subscript", "value" => "abc");
    check_org_aot_element!("x_{a{b}c}\n", "subscript", "value" => "a{b}c");
    check_org_aot_element!("x^2\n", "superscript", "value" => "2");
    check_org_aot_element!("x^*\n", "superscript", "value" => "*");
}

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

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "org_script_aot::configured_inline_script_policy_reaches_scheme_aot_helper",
        configured_inline_script_policy_reaches_scheme_aot_helper,
    ),
    (
        "org_script_aot::scheme_script_objects_project_with_source_backed_values",
        scheme_script_objects_project_with_source_backed_values,
    ),
    (
        "org_script_aot::script_guards_do_not_consume_identifiers_or_markup",
        script_guards_do_not_consume_identifiers_or_markup,
    ),
];
