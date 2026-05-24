use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

#[test]
fn eval_cli_renders_named_block_plan_without_running_code() {
    let dir = test_dir("eval-plan");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("notes.org"), eval_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["eval", "plan", "verify", "notes.org"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("orgize eval plan"), "stdout: {stdout}");
    assert!(stdout.contains("name: verify"), "stdout: {stdout}");
    assert!(stdout.contains("language: bash"), "stdout: {stdout}");
    assert!(
        stdout.contains("results: output replace"),
        "stdout: {stdout}"
    );
}

#[test]
fn eval_cli_patch_writes_results_without_executing_code() {
    let dir = test_dir("eval-patch-write");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("notes.org");
    fs::write(&path, eval_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "eval",
            "patch",
            "--write",
            "--stdout",
            "ok",
            "--exit-code",
            "0",
            "verify",
            "notes.org",
        ])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("kind: insert"), "stdout: {stdout}");
    assert!(stdout.contains("written: true"), "stdout: {stdout}");
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo should-not-run
#+END_SRC

#+RESULTS: verify
: ok
"#
    );
}

fn eval_fixture() -> &'static str {
    r#"#+NAME: verify
#+BEGIN_SRC bash :results output replace
echo should-not-run
#+END_SRC
"#
}

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        stdout(output),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
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
