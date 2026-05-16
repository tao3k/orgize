use orgize::fmt::{format_org, FormatOptions};

#[test]
fn fmt_preserves_file_and_attachment_links_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(file_and_attachment_links_fmt_fixture()));
}

fn file_and_attachment_links_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/file-and-attachment-links.org")
}

fn format_snapshot(source: &str) -> String {
    let formatted = format_org(source, &FormatOptions::default());
    let reformatted = format_org(&formatted.output, &FormatOptions::default());

    format!(
        "changed: {}\nidempotent: {}\noutput:\n{}",
        formatted.changed,
        formatted.output == reformatted.output,
        formatted.output
    )
}
