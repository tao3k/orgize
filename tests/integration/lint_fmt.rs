use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use orgize::lint::{lint_org, lint_org_with_options, LintOptions};

#[test]
fn cli_rejects_invalid_path_arguments_with_snapshot() {
    let dir = test_dir("invalid-paths");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("notes.txt"), skip_text_fmt_fixture()).unwrap();

    let missing_fmt = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "missing.org"])
        .output()
        .unwrap();
    let non_org_fmt = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "notes.txt"])
        .output()
        .unwrap();
    let missing_lint = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["lint", "missing.org"])
        .output()
        .unwrap();
    let non_org_lint = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["lint", "notes.txt"])
        .output()
        .unwrap();

    insta::assert_snapshot!(format!(
        "fmt missing:\n{}\nfmt non-org:\n{}\nlint missing:\n{}\nlint non-org:\n{}",
        command_snapshot(missing_fmt),
        command_snapshot(non_org_fmt),
        command_snapshot(missing_lint),
        command_snapshot(non_org_lint)
    ));
}

#[test]
fn lint_reports_semantic_and_uniqueness_findings_as_compact_snapshot() {
    let source = semantic_and_uniqueness_lint_fixture();
    let report = lint_org(source);

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_compact_text("fixture.org", source)
    ));
}

#[test]
fn lint_reports_semantic_and_uniqueness_findings_as_text_snapshot() {
    let report = lint_org(semantic_and_uniqueness_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_semantic_and_uniqueness_findings_as_json_snapshot() {
    let report = lint_org(semantic_and_uniqueness_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_json_file("fixture.org")
    ));
}

#[test]
fn lint_checks_include_paths_with_snapshot() {
    let dir = test_dir("lint-include-paths");
    fs::create_dir_all(dir.join("folder")).unwrap();
    fs::write(dir.join("present.org"), "* Present\n").unwrap();

    let report = lint_org_with_options(
        include_paths_lint_fixture(),
        &LintOptions {
            include_base_dir: Some(dir),
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_missing_macro_definitions_with_snapshot() {
    let report = lint_org(missing_macro_definitions_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_duplicate_macro_definitions_with_snapshot() {
    let report = lint_org(duplicate_macro_definitions_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_link_abbreviation_definition_issues_with_snapshot() {
    let report = lint_org(link_abbreviation_definition_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_supported_options_keyword_issues_with_snapshot() {
    let report = lint_org(supported_options_keyword_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_todo_declaration_issues_with_snapshot() {
    let report = lint_org(todo_declaration_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_reports_priority_property_issues_with_snapshot() {
    let report = lint_org(priority_property_issues_lint_fixture());

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("fixture.org")
    ));
}

#[test]
fn lint_cli_compact_stdin_output_is_snapshotted() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["lint"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(semantic_and_uniqueness_lint_fixture().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn lint_cli_text_stdin_output_is_snapshotted() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["lint", "--format", "text"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(semantic_and_uniqueness_lint_fixture().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    insta::assert_snapshot!(command_snapshot(output));
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
        .write_all(semantic_and_uniqueness_lint_fixture().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn lint_cli_checks_include_paths_relative_to_file_with_snapshot() {
    let dir = test_dir("lint-cli-include-paths");
    fs::create_dir_all(dir.join("notes/folder")).unwrap();
    fs::write(dir.join("notes/present.org"), "* Present\n").unwrap();
    fs::write(dir.join("notes/main.org"), include_paths_lint_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["lint", "--json", "notes/main.org"])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn lint_cli_directory_path_output_is_snapshotted() {
    let dir = test_dir("lint-dir");
    fs::create_dir_all(dir.join("notes/nested")).unwrap();
    fs::write(
        dir.join("notes/a.org"),
        semantic_and_uniqueness_lint_fixture(),
    )
    .unwrap();
    fs::write(dir.join("notes/nested/clean.org"), "* Clean\n").unwrap();
    fs::write(dir.join("notes/skip.txt"), "[[fn:missing]]\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["lint", "--json", "notes"])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

fn semantic_and_uniqueness_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/semantic-and-uniqueness.org")
}

fn include_paths_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/include-paths.org")
}

fn missing_macro_definitions_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/missing-macro-definitions.org")
}

fn duplicate_macro_definitions_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/duplicate-macro-definitions.org")
}

fn link_abbreviation_definition_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/link-abbreviation-definition-issues.org")
}

fn supported_options_keyword_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/supported-options-keyword-issues.org")
}

fn todo_declaration_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/todo-declaration-issues.org")
}

fn priority_property_issues_lint_fixture() -> &'static str {
    include_str!("../fixtures/lint/priority-property-issues.org")
}

fn skip_text_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/skip.txt")
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
