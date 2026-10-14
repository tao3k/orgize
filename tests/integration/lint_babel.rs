use orgize::lint::lint_org;

fn lint_reports_babel_source_block_issues_with_snapshot() {
    let source = babel_source_block_issues_lint_fixture();
    let report = lint_org(source);

    let eval_header = report
        .findings
        .iter()
        .find(|finding| finding.code == "ORG022")
        .expect("eval-sensitive header finding");
    assert_eq!(eval_header.location.start.line, 2);
    let duplicate_name = report
        .findings
        .iter()
        .find(|finding| finding.code == "ORG020")
        .expect("duplicate source-block name finding");
    assert_eq!(duplicate_name.location.start.line, 7);

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_compact_text("fixture.org", source)
    ));
}

fn babel_source_block_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/babel-source-block-issues.org")
}

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[(
    "lint_babel::lint_reports_babel_source_block_issues_with_snapshot",
    lint_reports_babel_source_block_issues_with_snapshot,
)];
