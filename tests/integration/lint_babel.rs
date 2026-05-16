use orgize::lint::lint_org;

#[test]
fn lint_reports_babel_source_block_issues_with_snapshot() {
    let source = babel_source_block_issues_lint_fixture();
    let report = lint_org(source);

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_compact_text("fixture.org", source)
    ));
}

fn babel_source_block_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/babel-source-block-issues.org")
}
