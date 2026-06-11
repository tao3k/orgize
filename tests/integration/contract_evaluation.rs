use std::{fs, path::PathBuf, process::Command};

use orgize::{
    Org,
    ast::{
        OrgContractAssertionStatus, OrgContractEvaluationScope, OrgContractRegistry,
        evaluate_org_contract, org_contract_evaluation_to_json_value, parse_contract_reference,
        parse_contracts_from_document,
    },
};

#[test]
fn exposes_matched_ids_and_bindings() {
    let registry = contract_registry();
    let contract = registry
        .resolve(&parse_contract_reference("agent.evidence-link-task.v1"))
        .expect("contract should resolve");
    let source = contract_target_source();
    let document = Org::parse(source).document();
    let scope = OrgContractEvaluationScope::section(
        "Task A",
        vec!["Task A".to_string()],
        document.sections[0].ann.range,
    );

    let evaluation = evaluate_org_contract(&document, contract, scope);

    assert_eq!(evaluation.contract_id, "agent.evidence-link-task.v1");
    assert_eq!(evaluation.scope.kind_as_str(), "section");
    assert_eq!(evaluation.assertions.len(), 1);
    let assertion = &evaluation.assertions[0];
    assert_eq!(assertion.assertion_id, "task.evidence-has-link");
    assert_eq!(assertion.status, OrgContractAssertionStatus::Passed);
    assert_eq!(assertion.actual_count, 1);
    assert_eq!(assertion.matched_ids.len(), 1);
    assert_eq!(assertion.bindings["evidence"].len(), 1);
    assert!(assertion.message_template.is_some());
}

#[test]
fn json_exports_source_backed_trace() {
    let registry = contract_registry();
    let contract = registry
        .resolve(&parse_contract_reference("agent.evidence-link-task.v1"))
        .expect("contract should resolve");
    let document = Org::parse(contract_target_source()).document();
    let evaluation = evaluate_org_contract(
        &document,
        contract,
        OrgContractEvaluationScope::section(
            "Task A",
            vec!["Task A".to_string()],
            document.sections[0].ann.range,
        ),
    );

    let json = org_contract_evaluation_to_json_value(&evaluation);

    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["contractId"], "agent.evidence-link-task.v1");
    assert_eq!(json["scope"]["kind"], "section");
    assert_eq!(json["scope"]["outlinePath"][0], "Task A");
    assert_eq!(
        json["assertions"][0]["assertionId"],
        "task.evidence-has-link"
    );
    assert_eq!(json["assertions"][0]["status"], "passed");
    assert_eq!(json["assertions"][0]["actualCount"], 1);
    assert_eq!(json["assertions"][0]["expectation"]["kind"], "count");
    assert_eq!(json["assertions"][0]["expectation"]["operator"], ">=");
    assert_eq!(
        json["assertions"][0]["matchedIds"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        json["assertions"][0]["bindings"]["evidence"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(json.get("score").is_none());
    assert!(json.get("verdict").is_none());
}

#[test]
fn cli_trace_outputs_contract_evaluation_json_snapshot() {
    let dir = test_dir("contract-trace");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("contract.org"), contract_source()).unwrap();
    fs::write(
        dir.join("notes.org"),
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: ./contract.org#agent.evidence-link-task.v1
:END:
** Evidence
[[https://example.test][inside]]
* Task B
:PROPERTIES:
:CONTRACT_ORG: ./contract.org#agent.evidence-link-task.v1
:END:
** Evidence
No link here.
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "contract",
            "trace",
            "--org-contract-registry",
            "contract.org",
            "notes.org",
        ])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

fn contract_registry() -> OrgContractRegistry {
    let document = Org::parse(contract_source()).document();
    parse_contracts_from_document(&document, None)
}

fn contract_source() -> &'static str {
    r#"* evidence-link-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.evidence-link-task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** evidence-has-link
:PROPERTIES:
:ASSERT_ID: task.evidence-has-link
:SEVERITY: warning
:END:

#+BEGIN_SRC org-contract
let evidence = headline where child_of($scope) and property(:raw-value) = "Evidence"

assert count link where
  descendant_of(evidence)
>= 1
#+END_SRC

#+BEGIN_SRC jinja2 :name message
Task `{{ scope.title }}` must include at least one link under Evidence.
#+END_SRC
"#
}

fn contract_target_source() -> &'static str {
    r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.evidence-link-task.v1
:END:
** Evidence
[[https://example.test][inside]]
"#
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
