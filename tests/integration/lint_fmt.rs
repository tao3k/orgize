use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use orgize::{
    fmt::{format_org, FormatOptions},
    lint::lint_org,
};

#[test]
fn fmt_normalizes_source_with_snapshot() {
    let source = "* Heading  \r\nBody\t \n\n\n";
    let formatted = format_org(source, &FormatOptions::default());

    assert!(formatted.changed);
    insta::assert_snapshot!(formatted.output);
}

#[test]
fn fmt_cli_check_output_is_snapshotted() {
    let dir = test_dir("fmt-check");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("dirty.org"), "* Heading  \n\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "--check", "dirty.org"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn lint_reports_semantic_and_uniqueness_findings_as_text_snapshot() {
    let report = lint_org(lint_fixture());

    assert!(!report.is_clean());
    insta::assert_snapshot!(report.to_text("fixture.org"));
}

#[test]
fn lint_reports_semantic_and_uniqueness_findings_as_json_snapshot() {
    let report = lint_org(lint_fixture());

    assert!(!report.is_clean());
    insta::assert_snapshot!(report.to_json_file("fixture.org"));
}

#[test]
fn lint_cli_json_stdin_output_is_snapshotted() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["lint", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(lint_fixture().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success());
    insta::assert_snapshot!(command_snapshot(output));
}

fn lint_fixture() -> &'static str {
    r#"* First
:PROPERTIES:
:ID: shared
:END:
* Second
:PROPERTIES:
:ID: shared
:END:
[[fn:missing]]
"#
}

fn command_snapshot(output: std::process::Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status.code().unwrap_or_default(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    )
}

fn test_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("orgize-cli-tests")
        .join(format!("{name}-{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).unwrap();
    }
    path
}
