use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_check_output_is_snapshotted",
        command_snapshot(output),
    );
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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_check_stdin_output_is_snapshotted",
        command_snapshot(output),
    );
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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_stdin_aligns_tables_with_snapshot",
        command_snapshot(output),
    );
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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_check_directory_output_is_snapshotted",
        command_snapshot(output),
    );
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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_default_writes_file_with_snapshot",
        format!(
            "{}\nfile: {:?}",
            command_snapshot(output),
            fs::read_to_string(dir.join("notes.org")).unwrap()
        ),
    );
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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_multiple_files_write_with_snapshot",
        format!(
            "{}\none.org: {:?}\ntwo.org: {:?}",
            command_snapshot(output),
            fs::read_to_string(dir.join("one.org")).unwrap(),
            fs::read_to_string(dir.join("two.org")).unwrap()
        ),
    );
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

    assert_lint_fmt_snapshot(
        "integration_test__lint_fmt__fmt_cli_directory_path_writes_org_files_with_snapshot",
        format!(
            "{}\na.org: {:?}\nnested/b.org: {:?}\nskip.txt: {:?}",
            command_snapshot(output),
            fs::read_to_string(dir.join("notes/a.org")).unwrap(),
            fs::read_to_string(dir.join("notes/nested/b.org")).unwrap(),
            fs::read_to_string(dir.join("notes/skip.txt")).unwrap()
        ),
    );
}

fn assert_lint_fmt_snapshot(name: &str, value: String) {
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(name, value);
    });
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
