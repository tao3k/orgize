use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use orgize::{
    fmt::{format_org, FormatOptions},
    lint::{lint_org, lint_org_with_options, LintOptions},
};

#[test]
fn fmt_normalizes_source_with_snapshot() {
    let source = "* Heading  \r\nBody\t \n\n\n";
    insta::assert_snapshot!(format_snapshot(source));
}

#[test]
fn fmt_aligns_tables_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(table_with_block_fmt_fixture()));
}

#[test]
fn fmt_aligns_complex_table_lines_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(complex_table_alignment_fmt_fixture()));
}

#[test]
fn fmt_aligns_indented_tables_formulas_and_pipe_rules_with_snapshot() {
    insta::assert_snapshot!(format_snapshot(indented_table_formulas_fmt_fixture()));
}

#[test]
fn fmt_cli_check_output_is_snapshotted() {
    let dir = test_dir("fmt-check");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("dirty.org"), default_write_fmt_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "--check", "dirty.org"])
        .output()
        .unwrap();

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
        .write_all(stdin_table_fmt_fixture().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

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
        .write_all(stdin_table_fmt_fixture().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn fmt_cli_check_directory_output_is_snapshotted() {
    let dir = test_dir("fmt-check-dir");
    fs::create_dir_all(dir.join("notes/nested")).unwrap();
    fs::write(dir.join("notes/a.org"), directory_a_fmt_fixture()).unwrap();
    fs::write(dir.join("notes/nested/b.org"), directory_b_fmt_fixture()).unwrap();
    fs::write(dir.join("notes/skip.txt"), skip_text_fmt_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "--check", "notes"])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn fmt_cli_default_writes_file_with_snapshot() {
    let dir = test_dir("fmt-default-write");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("notes.org"), default_write_fmt_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "notes.org"])
        .output()
        .unwrap();

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
    fs::write(dir.join("one.org"), multiple_one_fmt_fixture()).unwrap();
    fs::write(dir.join("two.org"), multiple_two_fmt_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "one.org", "two.org"])
        .output()
        .unwrap();

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
    fs::write(dir.join("notes/a.org"), directory_a_fmt_fixture()).unwrap();
    fs::write(dir.join("notes/nested/b.org"), directory_b_fmt_fixture()).unwrap();
    fs::write(dir.join("notes/skip.txt"), skip_text_fmt_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["fmt", "notes"])
        .output()
        .unwrap();

    insta::assert_snapshot!(format!(
        "{}\na.org: {:?}\nnested/b.org: {:?}\nskip.txt: {:?}",
        command_snapshot(output),
        fs::read_to_string(dir.join("notes/a.org")).unwrap(),
        fs::read_to_string(dir.join("notes/nested/b.org")).unwrap(),
        fs::read_to_string(dir.join("notes/skip.txt")).unwrap()
    ));
}

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

fn table_with_block_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/table-with-block.org")
}

fn complex_table_alignment_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/complex-table-alignment.org")
}

fn indented_table_formulas_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/indented-table-formulas.org")
}

fn stdin_table_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/stdin-table.org")
}

fn default_write_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/default-write.org")
}

fn multiple_one_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/multiple-one.org")
}

fn multiple_two_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/multiple-two.org")
}

fn directory_a_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/directory-a.org")
}

fn directory_b_fmt_fixture() -> &'static str {
    include_str!("../fixtures/fmt/directory-b.org")
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

fn format_snapshot(source: &str) -> String {
    let formatted = format_org(source, &FormatOptions::default());
    let reformatted = format_org(&formatted.output, &FormatOptions::default());

    format!(
        "changed: {}\nidempotent: {}\noutput:\n{}",
        formatted.changed,
        formatted.output == reformatted.output,
        formatted.output
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
