//! Source-backed list fields projected from Scheme-AOT syntax tokens.

#[test]
fn scheme_list_items_project_source_backed_indentation_and_spacing() {
    let source = "- one\n  - child\n";
    let document =
        orgize::org_aot::parse_org_aot(source).expect("Scheme list strategy parses nested items");
    let items = document
        .records()
        .iter()
        .filter(|record| record.kind == "item")
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].field("bullet"), Some("- "));
    assert!(items[0].values("trivia").next().is_none());
    assert_eq!(items[1].field("bullet"), Some("- "));
    assert_eq!(items[1].values("trivia").collect::<Vec<_>>(), ["  "]);
    assert_eq!(document.syntax().to_string(), source);
}

#[test]
fn scheme_list_bullet_keeps_exact_horizontal_separator() {
    let source = "-   one\n  -\tchild\n";
    let document =
        orgize::org_aot::parse_org_aot(source).expect("Scheme list strategy parses exact spacing");
    let bullets = document
        .records()
        .iter()
        .filter(|record| record.kind == "item")
        .map(|record| record.field("bullet"))
        .collect::<Vec<_>>();
    assert_eq!(bullets, [Some("-   "), Some("-\t")]);
    assert_eq!(document.syntax().to_string(), source);
}
