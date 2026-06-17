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
fn capture_plan_renders_agent_plan_template_contract() {
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

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(
        stdout.contains("org-entry:\n* TODO Close Org plan recording loop :plan:"),
        "{stdout}"
    );
    assert!(stdout.contains(":CAPTURE_KIND: agentPlan"), "{stdout}");
    assert!(
        stdout.contains(":PLAN_CONTRACT: agent.execplan.v1"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            ":CONTRACT_ORG: [[languages/org/contracts/agent.execplan.v1.org][agent.execplan.v1]]"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains(":PLAN_PROJECT: current-project"),
        "{stdout}"
    );
    assert!(
        stdout.contains(":PLAN_SESSION: current-session"),
        "{stdout}"
    );
    assert!(stdout.contains(":PLAN_BRANCH: main"), "{stdout}");
    assert!(stdout.contains(":PLAN_SHARING: session"), "{stdout}");
    assert!(stdout.contains(":PLAN_STATUS: draft"), "{stdout}");
    assert!(
        stdout.contains(":PLAN_INTERFACE: asp org query"),
        "{stdout}"
    );
    assert!(
        stdout.contains(":MEMORY_SCOPE: project=current-project;session=current-session;plan=org-plan-recording;branch=main"),
        "{stdout}"
    );
    assert!(stdout.contains(":MEMORY_RECALL_K1: 20"), "{stdout}");
    assert!(stdout.contains(":MEMORY_RECALL_K2: 5"), "{stdout}");
    assert!(stdout.contains(":MEMORY_RECALL_LAMBDA: 0.30"), "{stdout}");
    assert!(stdout.contains(":MEMORY_MIN_SCORE: 0.12"), "{stdout}");
    assert!(
        stdout.contains(":MEMORY_MAX_CONTEXT_CHARS: 1200"),
        "{stdout}"
    );
    assert!(stdout.contains(":MEMORY_FEEDBACK_BIAS: 0.0"), "{stdout}");
    assert!(!stdout.contains(":MEMORY_ENGINE:"), "{stdout}");
    assert!(stdout.contains(":PLAN_ID: org-plan-recording"), "{stdout}");
    assert!(stdout.contains("** Goal"), "{stdout}");
    assert!(
        stdout.contains("Use Org as the source of truth for agent execution state."),
        "{stdout}"
    );
    assert!(stdout.contains("** Memory Context"), "{stdout}");
    assert!(
        stdout.contains("PLAN_SHARING controls recall visibility"),
        "{stdout}"
    );
    assert!(
        stdout.contains("MEMORY_RECALL_K1, MEMORY_RECALL_K2, MEMORY_RECALL_LAMBDA"),
        "{stdout}"
    );
    assert!(
        stdout.contains("MEMORY_FEEDBACK_BIAS is applied"),
        "{stdout}"
    );
    assert!(stdout.contains("** Plan"), "{stdout}");
    assert!(
        stdout.contains("*** TODO P0 Contract and template are explicit."),
        "{stdout}"
    );
    assert!(stdout.contains(":STEP_ID: P0"), "{stdout}");
    assert!(stdout.contains(":STEP_STATUS: pending"), "{stdout}");
    assert!(stdout.contains("** Evidence"), "{stdout}");
    assert!(
        stdout.contains(
            "asp org query --kind property --field key=PLAN_ID --field value=org-plan-recording --workspace . --view metadata"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "asp org query --kind property --field key=PLAN_SESSION --field value=<SESSION_ID> --workspace . --view metadata"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "asp org query --kind property --field key=PLAN_BRANCH --field value=<BRANCH_ID> --workspace . --view metadata"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "asp org query --kind property --field key=PLAN_SHARING --field value=project --workspace . --view metadata"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "asp org query --kind property --field key=STEP_STATUS --field value=pending --workspace . --view metadata"
        ),
        "{stdout}"
    );
    assert!(stdout.contains("** Receipts"), "{stdout}");
    assert!(stdout.contains("** State Query"), "{stdout}");
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
