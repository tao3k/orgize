use std::{fs, path::PathBuf};

use orgize::lint::{lint_org_with_options, LintOptions};

#[test]
fn lint_reports_attachment_path_issues_with_snapshot() {
    let dir = test_dir("lint-attachment-paths");
    fs::create_dir_all(dir.join("assets")).unwrap();
    fs::write(dir.join("assets/present.txt"), "ok\n").unwrap();

    let source = include_str!("../fixtures/lint/attachment-issues.org");
    let report = lint_org_with_options(
        source,
        &LintOptions {
            attachment_base_dir: Some(dir),
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("attachment-issues.org")
    ));
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
