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
fn fmt_aligns_tables_with_snapshot() {
    let source = r#"| Name | Count |
|---+---|
| a | 2 |
| longer | 100 |

#+begin_example
| this | block |
| stays | as-is |
#+end_example
"#;
    let formatted = format_org(source, &FormatOptions::default());

    assert!(formatted.changed);
    insta::assert_snapshot!(formatted.output);
}

#[test]
fn fmt_aligns_indented_tables_formulas_and_pipe_rules_with_snapshot() {
    let source = r#"  | Name | Count |
  |---|---|
  | a | 2 |
  | longer | 100 |
#+TBLFM: $2=vsum(@2..@3)
"#;
    let formatted = format_org(source, &FormatOptions::default());
    let reformatted = format_org(&formatted.output, &FormatOptions::default());

    assert!(formatted.changed);
    assert_eq!(formatted.output, reformatted.output);
    insta::assert_snapshot!(format!(
        "idempotent: {}\n{}",
        formatted.output == reformatted.output,
        formatted.output
    ));
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
fn fmt_cli_check_stdin_output_is_snapshotted() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["fmt", "--check"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"| A | Longer |\n| value | x |\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success());
    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn fmt_cli_stdin_aligns_tables_with_snapshot() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["fmt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"| A | Longer |\n| value | x |\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn fmt_cli_check_directory_output_is_snapshotted() {
    let dir = test_dir("fmt-check-dir");
    fs::create_dir_all(dir.join("notes/nested")).unwrap();
    fs::write(dir.join("notes/a.org"), "A  \n").unwrap();
    fs::write(dir.join("notes/nested/b.org"), "| A | B |\n| xx | y |\n").unwrap();
    fs::write(dir.join("notes/skip.txt"), "skip  \n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "--check", "notes"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn fmt_cli_default_writes_file_with_snapshot() {
    let dir = test_dir("fmt-default-write");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("notes.org"), "| A | B |\n| x | longer |\n\n\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "notes.org"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(format!(
        "{}\nfile: {:?}",
        command_snapshot(output),
        fs::read_to_string(dir.join("notes.org")).unwrap()
    ));
}

#[test]
fn fmt_cli_multiple_files_write_with_snapshot() {
    let dir = test_dir("fmt-multiple-files");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("one.org"), "One  \n").unwrap();
    fs::write(dir.join("two.org"), "| A | Longer |\n| value | x |\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "one.org", "two.org"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(format!(
        "{}\none.org: {:?}\ntwo.org: {:?}",
        command_snapshot(output),
        fs::read_to_string(dir.join("one.org")).unwrap(),
        fs::read_to_string(dir.join("two.org")).unwrap()
    ));
}

#[test]
fn fmt_cli_directory_path_writes_org_files_with_snapshot() {
    let dir = test_dir("fmt-dir-write");
    fs::create_dir_all(dir.join("notes/nested")).unwrap();
    fs::write(dir.join("notes/a.org"), "A  \n").unwrap();
    fs::write(dir.join("notes/nested/b.org"), "| A | B |\n| xx | y |\n").unwrap();
    fs::write(dir.join("notes/skip.txt"), "skip  \n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "notes"])
        .output()
        .unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(format!(
        "{}\na.org: {:?}\nnested/b.org: {:?}\nskip.txt: {:?}",
        command_snapshot(output),
        fs::read_to_string(dir.join("notes/a.org")).unwrap(),
        fs::read_to_string(dir.join("notes/nested/b.org")).unwrap(),
        fs::read_to_string(dir.join("notes/skip.txt")).unwrap()
    ));
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

#[test]
fn lint_cli_directory_path_output_is_snapshotted() {
    let dir = test_dir("lint-dir");
    fs::create_dir_all(dir.join("notes/nested")).unwrap();
    fs::write(dir.join("notes/a.org"), lint_fixture()).unwrap();
    fs::write(dir.join("notes/nested/clean.org"), "* Clean\n").unwrap();
    fs::write(dir.join("notes/skip.txt"), "[[fn:missing]]\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["lint", "--json", "notes"])
        .output()
        .unwrap();

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
