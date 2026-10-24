use std::{fs, path::PathBuf};

use orgize::lint::{LintOptions, lint_org, lint_org_with_options};

fn lint_reports_lifecycle_archive_issues_with_snapshot() {
    let report = lint_org(lifecycle_archive_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

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

fn external_lifecycle_heading_queries_use_scheme_aot_structure() {
    let dir = test_dir("lint-lifecycle-aot-heading");
    fs::write(
        dir.join("destination.org"),
        "#+sEq_ToDo: WAIT | DONE\n#+BeGiN_SrC text\n* Hidden\n#+eNd_sRc\n* WAIT [#A] Real :tag:\n",
    )
    .unwrap();
    let options = LintOptions {
        file_base_dir: Some(dir),
        ..LintOptions::default()
    };

    let existing = lint_org_with_options("#+ARCHIVE: destination.org::* Real\n", &options);
    assert!(
        !existing
            .findings
            .iter()
            .any(|finding| finding.code == "ORG018"),
        "{:#?}",
        existing.findings
    );

    let hidden = lint_org_with_options("#+ARCHIVE: destination.org::* Hidden\n", &options);
    assert!(hidden.findings.iter().any(|finding| {
        finding.code == "ORG018" && finding.message.contains("heading `* Hidden` was not found")
    }));

    let refile = lint_org_with_options(
        "* TODO Work\n:LOGBOOK:\n- Refiled on [2026-05-14 Thu] from [[file:destination.org::* Hidden][hidden]]\n:END:\n",
        &options,
    );
    assert!(refile.findings.iter().any(|finding| {
        finding.code == "ORG019" && finding.message.contains("heading `* Hidden` was not found")
    }));
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

pub(super) const NATIVE_CASES: &[(&str, fn())] = &[
    (
        "lint_lifecycle::lint_reports_lifecycle_archive_issues_with_snapshot",
        lint_reports_lifecycle_archive_issues_with_snapshot,
    ),
    (
        "lint_lifecycle::lint_reports_lifecycle_destination_issues_with_snapshot",
        lint_reports_lifecycle_destination_issues_with_snapshot,
    ),
    (
        "lint_lifecycle::external_lifecycle_heading_queries_use_scheme_aot_structure",
        external_lifecycle_heading_queries_use_scheme_aot_structure,
    ),
];
