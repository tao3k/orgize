use orgize::lint::lint_org;

#[test]
fn lint_reports_table_formula_issues_with_snapshot() {
    let source = table_formula_issues_lint_fixture();
    let report = lint_org(source);

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_compact_text("fixture.org", source)
    ));
}

fn table_formula_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/table-formula-issues.org")
}
