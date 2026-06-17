use std::{fs, path::PathBuf, process::Command};

#[test]
fn task_list_renders_active_tasks_from_org_files() {
    let dir = test_dir("task-list-active");
    let path = dir.join("tasks.org");
    fs::write(&path, task_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["task-list", "--text", "routing", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("[TASK_LIST]"), "{stdout}");
    assert!(stdout.contains("- TODO Active routing"), "{stdout}");
    assert!(stdout.contains("scheduled: <2026-05-14 Thu>"), "{stdout}");
    assert!(!stdout.contains("DONE Finished routing"), "{stdout}");
}

#[test]
fn task_list_done_view_renders_done_tasks() {
    let dir = test_dir("task-list-done");
    let path = dir.join("tasks.org");
    fs::write(&path, task_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["task-list", "--view", "done", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("- DONE Finished routing"), "{stdout}");
    assert!(stdout.contains("closed: [2026-05-13 Wed]"), "{stdout}");
    assert!(!stdout.contains("TODO Active routing"), "{stdout}");
}

fn task_fixture() -> &'static str {
    r#"* TODO Active routing :work:
SCHEDULED: <2026-05-14 Thu>
Routing implementation is active.
* DONE Finished routing :work:
CLOSED: [2026-05-13 Wed]
Completed routing work.
* TODO Archived routing :ARCHIVE:
Hidden unless archived view is requested.
"#
}

fn assert_success(output: &std::process::Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        stdout(output),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
