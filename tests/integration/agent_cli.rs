use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

#[test]
fn cli_agent_planning_renders_planning_cards() {
    let dir = test_dir("agent-planning");
    let path = dir.join("agent.org");
    fs::write(&path, agent_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args([
            "agent-planning",
            "--date",
            "2026-05-14",
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(
        stdout.contains("[PLAN006] Action: Scheduled task"),
        "{stdout}"
    );
    assert!(stdout.contains("task: Capability SDD"), "{stdout}");
}

#[test]
fn cli_sparse_tree_renders_match_cards() {
    let dir = test_dir("sparse-tree");
    let path = dir.join("agent.org");
    fs::write(&path, agent_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["sparse-tree", "--text", "routing", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(
        stdout.contains("[SPARSE001] Match: Capability SDD"),
        "{stdout}"
    );
    assert!(stdout.contains("body=routing evidence"), "{stdout}");
}

fn agent_fixture() -> &'static str {
    r#"* System SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: system
:SDD_STATUS: review
:SDD_CONCERN: Routing evidence should stay source-grounded.
:END:
** TODO Capability SDD :sdd:
SCHEDULED: <2026-05-14 Thu>
:PROPERTIES:
:ID: 018f3f9c-7a91-73b4-b3f2-12c4c4d80d77
:SDD_KIND: capability
:SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]
:SDD_CAPABILITY: semantic-routing
:SDD_STATUS: review
:END:
routing evidence lives here.
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
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
