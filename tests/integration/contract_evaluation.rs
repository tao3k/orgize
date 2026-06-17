use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

#[test]
fn org_link_reference_uses_relative_path_and_display_contract_id() {
    let document = Org::parse(contract_source()).document();
    let registry = parse_contracts_from_document(
        &document,
        Some(Path::new("contracts/execplan.v1.contract.org")),
    );

    let reference = parse_contract_reference(
        "[[contracts/execplan.v1.contract.org][agent.evidence-link-task.v1]]",
    );

    assert_eq!(
        reference.path.as_deref(),
        Some("contracts/execplan.v1.contract.org")
    );
    assert_eq!(
        reference.contract_id.as_deref(),
        Some("agent.evidence-link-task.v1")
    );
    assert!(registry.resolve(&reference).is_some());
    assert!(
        registry
            .resolve(&parse_contract_reference(
                "[[contracts/wrong.contract.org][agent.evidence-link-task.v1]]"
            ))
            .is_none()
    );
}

#[test]
fn execplan_template_satisfies_language_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&manifest_dir)
        .args([
            "contract",
            "trace",
            "--org-contract-registry",
            "tests/fixtures/contract_evaluation/contracts/agent.execplan.v1.org",
            "tests/fixtures/contract_evaluation/templates/agent.execplan.v1.org",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains(r#""contractId": "agent.execplan.v1""#));
    assert!(!stdout.contains(r#""status": "failed""#), "{stdout}");
}

#[test]
fn reflection_answer_contract_requires_nonempty_value_cell() {
    let registry = {
        let document = Org::parse(reflection_answer_contract_source()).document();
        parse_contracts_from_document(&document, None)
    };
    let contract = registry
        .resolve(&parse_contract_reference("agent.reflection-answers.v1"))
        .expect("contract should resolve");

    let answered_document = Org::parse(reflection_answered_source()).document();
    let answered = evaluate_org_contract(
        &answered_document,
        contract,
        OrgContractEvaluationScope::section(
            "Reflection Questions",
            vec!["Reflection Questions".to_string()],
            answered_document.sections[0].ann.range,
        ),
    );
    assert!(
        answered
            .assertions
            .iter()
            .all(|assertion| assertion.status == OrgContractAssertionStatus::Passed),
        "{answered:?}"
    );
    let answered_value = answered
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_id == "reflection-has-nonempty-answer")
        .expect("nonempty assertion");
    assert_eq!(answered_value.actual_count, 1);

    let empty_document = Org::parse(reflection_empty_value_source()).document();
    let empty = evaluate_org_contract(
        &empty_document,
        contract,
        OrgContractEvaluationScope::section(
            "Reflection Questions",
            vec!["Reflection Questions".to_string()],
            empty_document.sections[0].ann.range,
        ),
    );
    let empty_value = empty
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_id == "reflection-has-nonempty-answer")
        .expect("nonempty assertion");
    assert_eq!(empty_value.status, OrgContractAssertionStatus::Failed);
    assert_eq!(empty_value.actual_count, 0);
}

#[test]
fn org_contract_accepts_elisp_style_query_expression_with_binding() {
    let registry = {
        let document = Org::parse(query_expression_contract_source()).document();
        parse_contracts_from_document(&document, None)
    };
    let contract = registry
        .resolve(&parse_contract_reference("agent.query-expression.v1"))
        .expect("contract should resolve");
    let document = Org::parse(query_expression_target_source()).document();
    let evaluation = evaluate_org_contract(
        &document,
        contract,
        OrgContractEvaluationScope::section(
            "Evidence Loop",
            vec!["Evidence Loop".to_string()],
            document.sections[0].ann.range,
        ),
    );

    let assertion = evaluation
        .assertions
        .iter()
        .find(|assertion| assertion.assertion_id == "evidence-link-from-cell")
        .expect("evidence-link assertion");
    assert_eq!(assertion.status, OrgContractAssertionStatus::Passed);
    assert_eq!(assertion.actual_count, 1);
    assert_eq!(assertion.bindings["evidence"].len(), 1);
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

fn reflection_answer_contract_source() -> &'static str {
    r#"* reflection-answers
:PROPERTIES:
:CONTRACT_ID: agent.reflection-answers.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** has-question-table
:PROPERTIES:
:ASSERT_ID: reflection-has-question-table
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
assert exists table where descendant_of($scope)
#+END_SRC

** has-question-column
:PROPERTIES:
:ASSERT_ID: reflection-has-question-column
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
assert exists table-cell where descendant_of($scope) and summary(text) = "Question"
#+END_SRC

** has-value-column
:PROPERTIES:
:ASSERT_ID: reflection-has-value-column
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
assert exists table-cell where descendant_of($scope) and summary(text) = "Value"
#+END_SRC

** has-nonempty-answer
:PROPERTIES:
:ASSERT_ID: reflection-has-nonempty-answer
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table-cell :descendant-of $scope :column "Value" :header nil :nonempty t))
#+END_SRC
"#
}

fn reflection_answered_source() -> &'static str {
    r#"* Reflection Questions
:PROPERTIES:
:CONTRACT_ORG: agent.reflection-answers.v1
:END:

| Question | Value |
|----------+-------|
| What should reflection record? | It must answer with a nonempty Value cell. |
"#
}

fn reflection_empty_value_source() -> &'static str {
    r#"* Reflection Questions
:PROPERTIES:
:CONTRACT_ORG: agent.reflection-answers.v1
:END:

| Question | Value |
|----------+-------|
| What should reflection record? | |
"#
}

fn query_expression_contract_source() -> &'static str {
    r#"* query-expression-contract
:PROPERTIES:
:CONTRACT_ID: agent.query-expression.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** evidence-link-from-cell
:PROPERTIES:
:ASSERT_ID: evidence-link-from-cell
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(let ((evidence
       (table-cell :descendant-of $scope :column "Evidence" :header nil :nonempty t)))
  (assert count >= 1
    (link :descendant-of evidence)))
#+END_SRC
"#
}

fn query_expression_target_source() -> &'static str {
    r#"* Evidence Loop
:PROPERTIES:
:CONTRACT_ORG: agent.query-expression.v1
:END:

| Claim | Evidence |
|-------+----------|
| Ready | [[https://example.test][trace]] |
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
