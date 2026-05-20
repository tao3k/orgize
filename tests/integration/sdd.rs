use std::{fs, path::PathBuf, process::Command};

use orgize::{Org, lint::lint_org};

#[test]
fn sdd_status_projects_org_native_parent_edges() {
    let document = Org::parse(valid_sdd_fixture()).document();
    let status = document.sdd_status();

    assert_eq!(status.records.len(), 5);
    assert_eq!(status.records[0].kind.as_str(), "system");
    assert_eq!(status.records[1].kind.as_str(), "capability");
    assert_eq!(
        status.records[1]
            .parent
            .as_ref()
            .and_then(|parent| parent.label.as_deref()),
        Some("System SDD")
    );

    let rendered = status.to_compact_text("fixture.org");
    assert!(rendered.contains("[SDD] fixture.org"));
    assert!(rendered.contains("architecture nodes: 5"));
    assert!(rendered.contains("- system review: System SDD"));
    assert!(rendered.contains("- view review: Runtime View"));
    assert!(rendered.contains("- decision accepted: Rust-owned Scheduling Decision"));
    assert!(rendered.contains("parent: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11 (System SDD)"));
    assert!(rendered.contains("viewpoint: runtime"));
    assert!(rendered.contains("rationale: Rust has deterministic admission control boundaries."));
}

#[test]
fn sdd_lint_reports_identity_parent_kind_metadata_and_requirement_issues() {
    let source = r#"* System SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: system
:END:
** Broken capability :sdd:
:PROPERTIES:
:ID: not-a-stable-id
:SDD_KIND: capability
:END:
*** Requirement: Missing scenario
The system SHALL expose bad SDD evidence.
** Broken view :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: surprise
:SDD_PARENT: semantic-parent-only
:END:
** TODO Checklist-shaped SDD [0/1] :sdd:
:PROPERTIES:
:ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78
:SDD_KIND: decision
:SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]
:SDD_RATIONALE: This is present so ORG036 isolates task-state misuse.
:END:
- [ ] Implement this plan step.
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
    assert!(codes.contains(&"ORG036"));
    assert!(codes.contains(&"ORG037"));

    let compact = report.to_compact_text("fixture.org", source);
    assert!(compact.contains("SDD system node `System SDD` is missing architecture metadata"));
    assert!(compact.contains("SDD node `Broken capability` has malformed ID"));
    assert!(compact.contains("SDD node `Broken capability` is missing SDD_PARENT"));
    assert!(compact.contains("SDD node `Broken view` has unsupported SDD_KIND"));
    assert!(compact.contains("SDD requirement `Requirement: Missing scenario`"));
    assert!(compact.contains("SDD headings must not carry TODO state"));
    assert!(compact.contains("SDD headings must not own direct task checklists"));
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
    assert!(stdout.contains("architecture nodes: 5"));
    assert!(stdout.contains("- audit review: Precision Audit"));
}

fn valid_sdd_fixture() -> &'static str {
    r#"* System SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11
:SDD_KIND: system
:SDD_STATUS: review
:SDD_CONCERN: OCR pipeline boundary, precision, and latency.
:END:
** Capability SDD :sdd:
:PROPERTIES:
:ID: 018f3f9c-7a91-73b4-b3f2-12c4c4d80d77
:SDD_KIND: capability
:SDD_PARENT: [[id:018f3f9c-8d3e-7b2a-9c91-4f5b2e7a2c11][System SDD]]
:SDD_CAPABILITY: polyglot-ocr-routing
:SDD_STATUS: review
:END:
*** Requirement: Architecture calibration
The system SHALL keep implementation plans linked to accepted design decisions.
**** Scenario: Agent inspects design drift
- WHEN an Agent queries SDD status
- THEN orgize SHALL expose architecture nodes, parent edges, concerns, and rationale.
** Runtime View :sdd:
:PROPERTIES:
:ID: 018f3f9c-55a2-70c0-98db-7ac2c4d80d78
:SDD_KIND: view
:SDD_PARENT: [[id:018f3f9c-7a91-73b4-b3f2-12c4c4d80d77][Capability SDD]]
:SDD_VIEWPOINT: runtime
:SDD_CONCERN: shard scheduling, backend routing, and fallback gates
:SDD_SLUG: runtime-view
:SDD_STATUS: review
:END:
** Rust-owned Scheduling Decision :sdd:
:PROPERTIES:
:ID: 018f3f9c-4242-72d0-a51d-0ac2c4d80d79
:SDD_KIND: decision
:SDD_PARENT: [[id:018f3f9c-55a2-70c0-98db-7ac2c4d80d78][Runtime View]]
:SDD_RATIONALE: Rust has deterministic admission control boundaries.
:SDD_STATUS: accepted
:END:
** Precision Audit :sdd:
:PROPERTIES:
:ID: 018f3f9c-4242-72d0-a51d-0ac2c4d80d80
:SDD_KIND: audit
:SDD_PARENT: [[id:018f3f9c-7a91-73b4-b3f2-12c4c4d80d77][Capability SDD]]
:SDD_CONCERN: OCR profile changes must not regress frozen precision evidence.
:SDD_STATUS: review
:END:
"#
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
