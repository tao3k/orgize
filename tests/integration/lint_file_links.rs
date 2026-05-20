use std::{fs, path::PathBuf};

use orgize::lint::{LintOptions, lint_org_with_options};

#[test]
fn lint_reports_file_link_path_issues_with_snapshot() {
    let dir = test_dir("lint-file-link-paths");
    fs::write(dir.join("present.org"), "* Present\n").unwrap();
    fs::create_dir_all(dir.join("directory")).unwrap();

    let source = r#"[[file:present.org]]
[[file:missing.org::*Heading]]
[[file:directory]]
[[file:]]
[[file:/ssh:host:/remote.org]]
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
        report.to_text("file-link-issues.org")
    ));
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
