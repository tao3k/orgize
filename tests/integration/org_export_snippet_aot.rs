//! Scheme-owned Org export snippets projected through the Rowan Element graph.

macro_rules! check_org_export_snippet {
    ($record:expr, $source:expr, $backend:expr, $value:expr, $literal:expr) => {{
        let record = $record;
        let range = record.range;
        assert_eq!(record.kind, "export-snippet");
        assert_eq!(record.field("backend"), Some($backend));
        assert_eq!(record.field("value"), $value);
        assert_eq!(
            &$source[usize::from(range.start())..usize::from(range.end())],
            $literal
        );
    }};
}

#[test]
fn scheme_export_snippets_keep_backend_value_and_source_spans() {
    let source = "go @@html:<b>x</b>@@ and @@-:@@\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("Scheme export snippets parse");
    assert_eq!(document.syntax().to_string(), source);
    let snippets = document
        .records()
        .iter()
        .filter(|record| record.kind == "export-snippet")
        .collect::<Vec<_>>();
    assert_eq!(snippets.len(), 2);
    check_org_export_snippet!(
        snippets[0],
        source,
        "html",
        Some("<b>x</b>"),
        "@@html:<b>x</b>@@"
    );
    check_org_export_snippet!(snippets[1], source, "-", Some(""), "@@-:@@");
}

#[test]
fn scheme_export_snippets_reject_malformed_and_opaque_context() {
    let source = "@@:x@@ @@h_t:x@@ @@html:x@\n#+begin_src text\n@@html:hidden@@\n#+end_src\n";
    let document = orgize::org_aot::parse_org_aot(source).expect("malformed snippets parse");
    assert_eq!(document.syntax().to_string(), source);
    assert_eq!(
        document
            .records()
            .iter()
            .filter(|record| record.kind == "export-snippet")
            .count(),
        0
    );
}
