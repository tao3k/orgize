use std::{fs, path::PathBuf, process::Command};

#[test]
fn capture_plan_renders_reviewable_plan_without_writing_org_file() {
    let dir = test_dir("capture-plan");
    let plan_path = dir.join("PLANS.org");

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "capture-plan",
            "--kind",
            "task",
            "--title",
            "Record ASP org plan",
            "--body",
            "Use asp org capture-plan before applying an Org edit.",
            "--target-file",
            "PLANS.org",
            "--outline",
            "Plans/Active",
            "--tag",
            "plan",
            "--property",
            "PLAN_ID=asp-org-recording",
        ])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(
        stdout.contains("[CAPTURE_PLAN] orgize capture-plan"),
        "{stdout}"
    );
    assert!(stdout.contains("target: outlinePath"), "{stdout}");
    assert!(stdout.contains("target-file: PLANS.org"), "{stdout}");
    assert!(stdout.contains("outline: Plans / Active"), "{stdout}");
    assert!(stdout.contains("requires-confirmation: true"), "{stdout}");
    assert!(stdout.contains("- writeLock:"), "{stdout}");
    assert!(stdout.contains("- outlinePathResolution:"), "{stdout}");
    assert!(stdout.contains("- nonMutating:"), "{stdout}");
    assert!(stdout.contains("- callerInterpreted:"), "{stdout}");
    assert!(!stdout.contains("Agent/caller"), "{stdout}");
    assert!(!stdout.contains("agentInterpreted"), "{stdout}");
    assert!(
        stdout.contains("org-entry:\n* TODO Record ASP org plan :plan:"),
        "{stdout}"
    );
    assert!(stdout.contains(":CAPTURE_KIND: task"), "{stdout}");
    assert!(stdout.contains(":PLAN_ID: asp-org-recording"), "{stdout}");
    assert!(
        stdout.contains("capture-plan performed no write"),
        "{stdout}"
    );
    assert!(
        !plan_path.exists(),
        "capture-plan must not create {}",
        plan_path.display()
    );
}

#[test]
fn capture_plan_rejects_domain_specific_agent_plan_kind() {
    let dir = test_dir("agent-plan-template");
    let plan_path = dir.join("PLANS.org");

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "capture-plan",
            "--kind",
            "agent-plan",
            "--title",
            "Close Org plan recording loop",
            "--body",
            "Use Org as the source of truth for agent execution state.",
            "--target-file",
            "PLANS.org",
            "--outline",
            "Plans/Active",
            "--tag",
            "plan",
            "--property",
            "PLAN_ID=org-plan-recording",
        ])
        .output()
        .unwrap();

    assert_ne!(
        output.status.code(),
        Some(0),
        "agent-plan must not be a built-in capture kind"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported capture kind `agent-plan`"),
        "{stderr}"
    );
    assert!(
        !plan_path.exists(),
        "capture-plan must not create {}",
        plan_path.display()
    );
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
