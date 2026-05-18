use std::{fs, path::PathBuf, process::Command};

use orgize::{lint::lint_org, Org};

#[test]
fn sdd_status_projects_org_native_parent_edges() {
    let document = Org::parse(valid_sdd_fixture()).document();
    let status = document.sdd_status();

    assert_eq!(status.records.len(), 3);
    assert_eq!(status.records[0].kind.as_str(), "program");
    assert_eq!(status.records[1].kind.as_str(), "capability");
    assert_eq!(
        status.records[1]
            .parent
            .as_ref()
            .and_then(|parent| parent.label.as_deref()),
        Some("Program SDD")
    );

    let rendered = status.to_compact_text("fixture.org");
    assert!(rendered.contains("[SDD] fixture.org"));
    assert!(rendered.contains("nodes: 3"));
    assert!(rendered.contains("- program active: Program SDD"));
    assert!(rendered.contains("parent: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11 (Program SDD)"));
}

#[test]
fn sdd_lint_reports_identity_parent_kind_and_requirement_issues() {
    let source = r#"* Program SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: program
:END:
** Broken capability :sdd:
:PROPERTIES:
:ID: not-a-stable-id
:SDD_KIND: capability
:END:
*** Requirement: Missing scenario
The system SHALL expose bad SDD evidence.
** Broken change :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: surprise
:SDD_PARENT: semantic-parent-only
:END:
"#;

    let report = lint_org(source);
    let codes = report
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"ORG031"));
    assert!(codes.contains(&"ORG032"));
    assert!(codes.contains(&"ORG033"));
    assert!(codes.contains(&"ORG034"));
    assert!(codes.contains(&"ORG035"));

    let compact = report.to_compact_text("fixture.org", source);
    assert!(compact.contains("SDD node `Broken capability` has malformed ID"));
    assert!(compact.contains("SDD node `Broken capability` is missing SDD_PARENT"));
    assert!(compact.contains("SDD node `Broken change` has unsupported SDD_KIND"));
    assert!(compact.contains("SDD requirement `Requirement: Missing scenario`"));
}

#[test]
fn cli_sdd_status_renders_compact_projection() {
    let dir = test_dir("sdd-status");
    let path = dir.join("sdd.org");
    fs::write(&path, valid_sdd_fixture()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["sdd", "status", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("[SDD]"));
    assert!(stdout.contains("nodes: 3"));
    assert!(stdout.contains("- change active: Change SDD"));
}

fn valid_sdd_fixture() -> &'static str {
    r#"* Program SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: program
:SDD_STATUS: active
:END:
** Capability SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-7a91-73b4-b3f2-12c4c4d80d77
:SDD_KIND: capability
:SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][Program SDD]]
:SDD_CAPABILITY: agent-planning
:SDD_STATUS: active
:END:
*** Requirement: Child SDD dispatch
The system SHALL allow parent SDD nodes to dispatch child SDD nodes.
**** Scenario: Agent resumes child SDD
- WHEN an Agent queries active SDD work
- THEN orgize SHALL expose parent and child status.
** Change SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78
:SDD_KIND: change
:SDD_PARENT: [[id:018f3f9c-7a91-73b4-b3f2-12c4c4d80d77][Capability SDD]]
:SDD_SLUG: org-native-sdd
:SDD_STATUS: active
:END:
"#
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
