use orgize::Org;

fn html_export_preserves_citation_raw_text() {
    let html = Org::parse("See [cite:@doe2020].").to_html();

    assert_eq!(
        html,
        "<main><section><p>See [cite:@doe2020].</p></section></main>"
    );
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[(
    "semantic_ast::html_export_preserves_citation_raw_text::html_export_preserves_citation_raw_text",
    html_export_preserves_citation_raw_text,
)];
