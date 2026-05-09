use super::*;

#[test]
fn html_export_preserves_citation_raw_text() {
    let html = Org::parse("See [cite:@doe2020].").to_html();

    assert_eq!(
        html,
        "<main><section><p>See [cite:@doe2020].</p></section></main>"
    );
}
