use std::{fs, path::PathBuf};

use orgize::lint::{LintOptions, lint_org, lint_org_with_options};

#[test]
fn lint_reports_lifecycle_archive_issues_with_snapshot() {
    let report = lint_org(lifecycle_archive_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_lifecycle_destination_issues_with_snapshot() {
    let dir = test_dir("lint-lifecycle-destinations");
    fs::write(dir.join("archive.org"), "* Existing\n").unwrap();
    fs::write(dir.join("old.org"), "* Old\n").unwrap();

    let source = r#"#+ARCHIVE: archive.org::* Missing
* TODO Active
:PROPERTIES:
:ARCHIVE: missing-archive.org::* Existing
:END:
:LOGBOOK:
- Refiled on [2026-05-14 Thu] from [[file:old.org::*Missing][missing]]
:END:
"#;
    let report = lint_org_with_options(
        source,
        &LintOptions {
            file_base_dir: Some(dir),
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("lifecycle-destination-issues.org")
    ));
}

fn lifecycle_archive_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/lifecycle-archive-issues.org")
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
