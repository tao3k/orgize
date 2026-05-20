use orgize::fmt::{FormatOptions, format_org};

#[test]
fn fmt_normalizes_source_with_snapshot() {
    let source = "* Heading  \r\nBody\t \n\n\n";
    insta::assert_snapshot!(format_snapshot(source));
}

#[test]
fn fmt_aligns_tables_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(table_with_block_fmt_fixture()));
}

#[test]
fn fmt_aligns_complex_table_lines_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(complex_table_alignment_fmt_fixture()));
}

#[test]
fn fmt_aligns_indented_tables_formulas_and_pipe_rules_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(indented_table_formulas_fmt_fixture()));
}

#[test]
fn fmt_aligns_official_style_tables_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(official_style_table_alignment_fmt_fixture()));
}

fn table_with_block_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/table-with-block.org")
}

fn complex_table_alignment_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/complex-table-alignment.org")
}

fn indented_table_formulas_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/indented-table-formulas.org")
}

fn official_style_table_alignment_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/official-style-table-alignment.org")
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
