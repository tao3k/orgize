use orgize::fmt::{FormatOptions, format_org};

fn fmt_normalizes_source_with_snapshot() {
    let source = "* Heading  \r\nBody\t \n\n\n";
    insta::assert_snapshot!(format_snapshot(source));
}

fn fmt_aligns_tables_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(table_with_block_fmt_fixture()));
}

fn fmt_aligns_complex_table_lines_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(complex_table_alignment_fmt_fixture()));
}

fn fmt_aligns_indented_tables_formulas_and_pipe_rules_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(indented_table_formulas_fmt_fixture()));
}

fn fmt_aligns_official_style_tables_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(official_style_table_alignment_fmt_fixture()));
}

fn fmt_uses_scheme_cells_for_escaped_table_pipes() {
    insta::assert_snapshot!(
        format_org("| Name | Value |\n|---+---|\n| a\\|b | 1 |\n", &FormatOptions::default()).output,
        @r###"
| Name | Value |
|------+-------|
| a\|b | 1     |
"###
    );
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

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "fmt_table::fmt_normalizes_source_with_snapshot",
        fmt_normalizes_source_with_snapshot,
    ),
    (
        "fmt_table::fmt_aligns_tables_with_snapshot",
        fmt_aligns_tables_with_snapshot,
    ),
    (
        "fmt_table::fmt_aligns_complex_table_lines_with_snapshot",
        fmt_aligns_complex_table_lines_with_snapshot,
    ),
    (
        "fmt_table::fmt_aligns_indented_tables_formulas_and_pipe_rules_with_snapshot",
        fmt_aligns_indented_tables_formulas_and_pipe_rules_with_snapshot,
    ),
    (
        "fmt_table::fmt_aligns_official_style_tables_with_snapshot",
        fmt_aligns_official_style_tables_with_snapshot,
    ),
    (
        "fmt_table::fmt_uses_scheme_cells_for_escaped_table_pipes",
        fmt_uses_scheme_cells_for_escaped_table_pipes,
    ),
];
