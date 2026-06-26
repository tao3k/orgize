use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use orgize::{
    Org,
    ast::{
        OrgContractAssertionStatus, OrgContractEvaluationScope, OrgContractRegistry,
        evaluate_org_contract, org_contract_evaluation_to_json_value, parse_contract_reference,
        parse_contracts_from_document,
    },
};

const CONTRACT_ORG_SCOPE_CONTRACTS: &str = include_str!(
    "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/contracts.org"
);
const CONTRACT_ORG_SCOPE_NOTES: &str =
    include_str!("../unit/scenarios/contract_trace/contract_org_property_scope/inputs/notes.org");
const CONTRACT_ORG_SCOPE_KEYWORD_NOTES: &str = include_str!(
    "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/keyword-notes.org"
);
const CONTRACT_ORG_SCOPE_DUPLICATE_NOTES: &str = include_str!(
    "../unit/scenarios/contract_trace/contract_org_property_scope/inputs/duplicate-notes.org"
);

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
fn plain_text_summary_contains_matches_inline_rendered_text_split_by_subscript() {
    let contract_document = Org::parse(
        r#"
* skill-contract
:PROPERTIES:
:CONTRACT_ID: asp.skill.test.v1
:CONTRACT_SCOPE: document
:END:

** Explicit capture command
:PROPERTIES:
:ASSERT_ID: asp.skill.capture.explicit-contract
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (plain-text :descendant-of $scope :summary-contains (value "asp org capture --contract CONTRACT_ID")))
#+END_SRC
"#,
    )
    .document();
    let registry = parse_contracts_from_document(&contract_document, None);
    let contract = registry
        .resolve(&parse_contract_reference("asp.skill.test.v1"))
        .expect("contract should resolve");
    let target = Org::parse(
        r#"
* Skill
Use asp org capture --contract CONTRACT_ID before writing.
"#,
    )
    .document();

    let evaluation =
        evaluate_org_contract(&target, contract, OrgContractEvaluationScope::document());

    assert_eq!(evaluation.assertions.len(), 1);
    let assertion = &evaluation.assertions[0];
    assert_eq!(assertion.status, OrgContractAssertionStatus::Passed);
    assert_eq!(assertion.actual_count, 1);
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
fn cli_trace_evaluates_multiple_contract_org_bindings_on_same_scope() {
    let dir = test_dir("contract-trace-multiple-contracts");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("contracts.org"),
        r#"* generic-skill-v1
:PROPERTIES:
:CONTRACT_ID: skill.generic.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:
** has-root-heading
:PROPERTIES:
:ASSERT_ID: skill.has-root-heading
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (headline :at $scope))
#+END_SRC
* asp-skill-v1
:PROPERTIES:
:CONTRACT_ID: asp.skill.test.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:
** has-evidence-section
:PROPERTIES:
:ASSERT_ID: asp.skill.has-evidence-section
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (headline :child-of $scope :summary (title "Evidence")))
#+END_SRC
"#,
    )
    .unwrap();
    fs::write(
        dir.join("skill.org"),
        r#"* ASP Org
:PROPERTIES:
:CONTRACT_ORG: skill.generic.v1
:CONTRACT_ORG: asp.skill.test.v1
:END:
** Evidence
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "contract",
            "trace",
            "--org-contract-registry",
            "contracts.org",
            "skill.org",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains(r#""contractId": "skill.generic.v1""#),
        "{stdout}"
    );
    assert!(
        stdout.contains(r#""contractId": "asp.skill.test.v1""#),
        "{stdout}"
    );
    assert!(!stdout.contains(r#""status": "failed""#), "{stdout}");
}

#[test]
fn cli_trace_distinguishes_document_and_heading_contract_org_property_scope() {
    let dir = test_dir("contract-trace-document-heading-scope");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("contracts.org"), CONTRACT_ORG_SCOPE_CONTRACTS).unwrap();
    fs::write(dir.join("notes.org"), CONTRACT_ORG_SCOPE_NOTES).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "contract",
            "trace",
            "--org-contract-registry",
            "contracts.org",
            "notes.org",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("trace JSON");
    let evaluations = json["files"][0]["evaluations"]
        .as_array()
        .expect("evaluations array");
    let actual_order = evaluations
        .iter()
        .map(|evaluation| {
            (
                evaluation["contractId"].as_str().unwrap_or_default(),
                evaluation["scope"]["kind"].as_str().unwrap_or_default(),
                evaluation["scope"]["title"].as_str().unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual_order,
        vec![
            ("document.scope.v1", "document", ""),
            ("document.contract-binding.v1", "document", ""),
            ("section.scope.v1", "section", "Task A"),
            ("section.scope.v1", "section", "Task A Child"),
            ("section.scope.v1", "section", "Task B"),
            ("section.override-title.v1", "section", "Override Parent"),
            ("section.override-title.v1", "section", "Override Child"),
        ],
        "{stdout}"
    );
    assert!(
        evaluations
            .iter()
            .flat_map(|evaluation| evaluation["assertions"].as_array().unwrap())
            .all(|assertion| assertion["status"] == "passed"),
        "{stdout}"
    );
}

#[test]
fn contract_org_property_scope_fixture_stays_in_millisecond_budget() {
    let scenario_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("unit")
        .join("scenarios")
        .join("contract_trace")
        .join("contract_org_property_scope");
    let benchmark = rust_lang_project_harness::validate_rust_scenario_benchmark(&scenario_root)
        .expect("validate contract trace property scope scenario benchmark");
    assert_eq!(
        benchmark.status,
        rust_lang_project_harness::RustScenarioBenchmarkStatus::Pass,
        "{:?}",
        benchmark.violations
    );
    let max_total = benchmark.benchmark.max_total.as_duration();

    let started_at = Instant::now();
    let contract_document = Org::parse(CONTRACT_ORG_SCOPE_CONTRACTS).document();
    let registry = parse_contracts_from_document(&contract_document, None);
    let notes_document = Org::parse(CONTRACT_ORG_SCOPE_NOTES).document();
    let document_contract = registry
        .resolve(&parse_contract_reference("document.scope.v1"))
        .expect("document contract");
    let document_binding_contract = registry
        .resolve(&parse_contract_reference("document.contract-binding.v1"))
        .expect("document binding contract");
    let section_contract = registry
        .resolve(&parse_contract_reference("section.scope.v1"))
        .expect("section contract");
    let override_contract = registry
        .resolve(&parse_contract_reference("section.override-title.v1"))
        .expect("override contract");
    let task_a = &notes_document.sections[0];
    let task_a_child = &task_a.subsections[1];
    let task_b = &notes_document.sections[1];
    let override_parent = &notes_document.sections[2];
    let override_child = &override_parent.subsections[0];
    let evaluations = vec![
        evaluate_org_contract(
            &notes_document,
            document_contract,
            OrgContractEvaluationScope::document(),
        ),
        evaluate_org_contract(
            &notes_document,
            document_binding_contract,
            OrgContractEvaluationScope::document(),
        ),
        evaluate_org_contract(
            &notes_document,
            section_contract,
            OrgContractEvaluationScope::section(
                "Task A",
                vec!["Task A".to_string()],
                task_a.ann.range,
            ),
        ),
        evaluate_org_contract(
            &notes_document,
            section_contract,
            OrgContractEvaluationScope::section(
                "Task A Child",
                vec!["Task A".to_string(), "Task A Child".to_string()],
                task_a_child.ann.range,
            ),
        ),
        evaluate_org_contract(
            &notes_document,
            section_contract,
            OrgContractEvaluationScope::section(
                "Task B",
                vec!["Task B".to_string()],
                task_b.ann.range,
            ),
        ),
        evaluate_org_contract(
            &notes_document,
            override_contract,
            OrgContractEvaluationScope::section(
                "Override Parent",
                vec!["Override Parent".to_string()],
                override_parent.ann.range,
            ),
        ),
        evaluate_org_contract(
            &notes_document,
            override_contract,
            OrgContractEvaluationScope::section(
                "Override Child",
                vec!["Override Parent".to_string(), "Override Child".to_string()],
                override_child.ann.range,
            ),
        ),
    ];
    let elapsed = started_at.elapsed();

    assert!(
        elapsed < max_total,
        "contract property scope fixture exceeded {}ms gate: {elapsed:?}",
        max_total.as_millis()
    );
    let failed_assertions = evaluations
        .iter()
        .flat_map(|evaluation| &evaluation.assertions)
        .filter(|assertion| assertion.status == OrgContractAssertionStatus::Failed)
        .count();
    assert_eq!(failed_assertions, 0, "{evaluations:?}");
}

#[test]
fn cli_trace_rejects_contract_org_metadata_keyword_declarations() {
    let dir = test_dir("contract-trace-rejects-contract-org-keyword");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("contract.org"), CONTRACT_ORG_SCOPE_CONTRACTS).unwrap();
    fs::write(dir.join("notes.org"), CONTRACT_ORG_SCOPE_KEYWORD_NOTES).unwrap();

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

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("CONTRACT_ORG must be declared in a property drawer"),
        "{stderr}"
    );
}

#[test]
fn cli_trace_rejects_duplicate_contract_org_bindings_on_same_scope() {
    let dir = test_dir("contract-trace-rejects-duplicate-contract-org");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("contract.org"), CONTRACT_ORG_SCOPE_CONTRACTS).unwrap();
    fs::write(dir.join("notes.org"), CONTRACT_ORG_SCOPE_DUPLICATE_NOTES).unwrap();

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

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(
        stderr.contains("duplicate CONTRACT_ORG `document.scope.v1` on the same scope"),
        "{stderr}"
    );
}

#[test]
fn cli_trace_resolves_org_link_contract_reference_relative_to_source_file() {
    let dir = test_dir("contract-trace-relative-org-link");
    fs::create_dir_all(dir.join("contracts")).unwrap();
    fs::create_dir_all(dir.join("templates")).unwrap();
    fs::write(
        dir.join("contracts").join("contract.org"),
        contract_source(),
    )
    .unwrap();
    fs::write(
        dir.join("templates").join("skill.org"),
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: [[../contracts/contract.org][agent.evidence-link-task.v1]]
:END:
** Evidence
[[https://example.test][inside]]
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "contract",
            "trace",
            "--org-contract-registry",
            "contracts/contract.org",
            "templates/skill.org",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains(r#""contractId": "agent.evidence-link-task.v1""#),
        "{stdout}"
    );
    assert!(!stdout.contains(r#""status": "failed""#), "{stdout}");
}

#[test]
fn cli_query_surface_outputs_agent_facing_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .args(["contract", "query-surface", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("query surface JSON");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["kind"], "org-elements-query-expression-surface");

    let guide = json["guide"].as_array().expect("guide array");
    assert!(
        guide.iter().any(|entry| entry
            .as_str()
            .is_some_and(|value| value.contains("lineage"))),
        "{stdout}"
    );
    assert!(
        guide.iter().any(|entry| entry
            .as_str()
            .is_some_and(|value| value.contains("secondary"))),
        "{stdout}"
    );

    let examples = json["examples"].as_array().expect("examples array");
    assert!(
        examples.iter().any(|entry| entry
            .as_str()
            .is_some_and(|value| value.contains("org-elements-query"))),
        "{stdout}"
    );
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
(let ((evidence
       (headline :child-of $scope :property (:raw-value "Evidence"))))
  (assert count >= 1
    (link :descendant-of evidence)))
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
(assert exists
  (table :descendant-of $scope))
#+END_SRC

** has-question-column
:PROPERTIES:
:ASSERT_ID: reflection-has-question-column
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table-cell :descendant-of $scope :text "Question"))
#+END_SRC

** has-value-column
:PROPERTIES:
:ASSERT_ID: reflection-has-value-column
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (table-cell :descendant-of $scope :text "Value"))
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
