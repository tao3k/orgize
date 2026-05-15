use orgize::lint::lint_org;

#[test]
fn lint_reports_lifecycle_archive_issues_with_snapshot() {
    let report = lint_org(lifecycle_archive_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

fn lifecycle_archive_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/lifecycle-archive-issues.org")
}
