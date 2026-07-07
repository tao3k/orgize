use std::{fs, path::PathBuf, process::Command, time::Instant};

use orgize::{
    Org,
    ast::parse_contracts_from_document,
    lint::{LintOptions, lint_org_with_options},
};

#[test]
fn lint_contract_org_query_assertion_scenario_has_snapshot() {
    let registry = contract_registry();
    let failure_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.task.v1
:END:
** Context
"#;
    let success_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.task.v1
:END:
** Goal
"#;
    let failure_report = lint_org_with_options(
        failure_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let success_report = lint_org_with_options(
        success_source,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "failure clean: {}\n{}\n\
         failure compact:\n{}\n\
         success clean: {}\n{}",
        failure_report.is_clean(),
        failure_report.to_text("contract-failure.org"),
        failure_report.to_compact_text("contract-failure.org", failure_source),
        success_report.is_clean(),
        success_report.to_text("contract-success.org")
    ));
}

#[test]
fn parse_contracts_preserves_split_query_and_expect_blocks() {
    let contract_fixture =
        include_str!("../unit/scenarios/lint_contract/org_query_assertion/inputs/contract.org");
    let document = Org::parse(contract_fixture).document();
    let registry = parse_contracts_from_document(&document, None);

    assert_eq!(registry.contracts.len(), 1);
    let contract = &registry.contracts[0];
    assert_eq!(contract.id, "agent.task.v1");
    assert_eq!(contract.assertions.len(), 1);
    let assertion = &contract.assertions[0];
    assert_eq!(assertion.id, "task.has-goal");
    assert_eq!(
        assertion.message.as_deref().map(str::trim),
        Some("Task `{{ scope.title }}` must contain a Goal section.")
    );
}

#[test]
fn parse_contracts_accepts_legacy_key_value_query_blocks() {
    let contract_fixture = r#"* agent-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** must-have-goal-section
:PROPERTIES:
:ASSERT_ID: task.has-goal
:SEVERITY: error
:END:

#+BEGIN_SRC org-elements-query
category = "section"
kind = "headline"
within = "$scope"
summary.title = "Goal"
#+END_SRC

#+BEGIN_SRC org-elements-expect
count >= 1
#+END_SRC
"#;
    let document = Org::parse(contract_fixture).document();
    let registry = parse_contracts_from_document(&document, None);

    assert_eq!(registry.contracts.len(), 1);
    let contract = &registry.contracts[0];
    assert_eq!(contract.assertions.len(), 1);
    assert_eq!(contract.assertions[0].id, "task.has-goal");
}

#[test]
fn lint_contract_binding_descendant_does_not_match_sibling_section() {
    let document = Org::parse(
        r#"* task-v1
:PROPERTIES:
:CONTRACT_ID: task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** acceptance-has-checklist
:PROPERTIES:
:ASSERT_ID: task.acceptance-has-checklist
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(let ((acceptance
       (headline :child-of $scope :summary (title "Acceptance"))))
  (assert count >= 1
    (and
      (item :descendant-of acceptance)
      (or
        (= (summary checkbox) "off")
        (= (summary checkbox) "on")
        (= (summary checkbox) "trans")))))
#+END_SRC
"#,
    )
    .document();
    let registry = parse_contracts_from_document(&document, None);
    let report = lint_org_with_options(
        r#"* TODO Task
:PROPERTIES:
:CONTRACT_ORG: task.v1
:END:

** Acceptance
No checklist here.

** Progress
- [ ] This sibling checklist must not satisfy Acceptance.
"#,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| finding.code)
            .collect::<Vec<_>>(),
        ["ORG044"]
    );
}

#[test]
fn lint_accepts_contract_org_when_assertion_query_matches() {
    let registry = contract_registry();
    let report = lint_org_with_options(
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.task.v1
:END:
** Goal
"#,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    assert!(report.is_clean(), "{}", report.to_text("fixture.org"));
}

#[test]
fn lint_reports_contract_org_query_assertion_failure_code() {
    let registry = contract_registry();
    let report = lint_org_with_options(
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.task.v1
:END:
** Context
"#,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    assert_eq!(
        report
            .findings
            .iter()
            .map(|finding| finding.code)
            .collect::<Vec<_>>(),
        ["ORG044"]
    );
}

#[test]
fn lint_contract_org_query_assertion_fixture_stays_in_millisecond_budget() {
    let scenario_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("unit")
        .join("scenarios")
        .join("lint_contract")
        .join("org_query_assertion");
    let benchmark = rust_lang_project_harness::validate_rust_scenario_benchmark(&scenario_root)
        .expect("validate org query assertion scenario benchmark");
    assert_eq!(
        benchmark.status,
        rust_lang_project_harness::RustScenarioBenchmarkStatus::Pass,
        "{:?}",
        benchmark.violations
    );
    let max_total = benchmark.benchmark.max_total.as_duration();
    let contract_fixture =
        include_str!("../unit/scenarios/lint_contract/org_query_assertion/inputs/contract.org");

    let started_at = Instant::now();
    let document = Org::parse(contract_fixture).document();
    let registry = parse_contracts_from_document(&document, None);
    let failure_report = lint_org_with_options(
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.task.v1
:END:
** Context
"#,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let elapsed = started_at.elapsed();

    assert_eq!(
        failure_report
            .findings
            .iter()
            .map(|finding| finding.code)
            .collect::<Vec<_>>(),
        ["ORG044"]
    );
    assert!(
        elapsed < max_total,
        "org query assertion fixture exceeded {}ms gate: {elapsed:?}",
        max_total.as_millis()
    );
}

#[test]
fn lint_contract_org_document_default_and_section_override_has_snapshot() {
    let registry = contract_registry();
    let source = r#"#+CONTRACT_ORG: agent.task.v1
* Task A
** Context
* Research Note
:PROPERTIES:
:CONTRACT_ORG: agent.research.v1
:END:
** Notes
"#;
    let report = lint_org_with_options(
        source,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}\ncompact:\n{}",
        report.is_clean(),
        report.to_text("contract-default-override.org"),
        report.to_compact_text("contract-default-override.org", source)
    ));
}

#[test]
fn lint_contract_org_elements_kind_queries_headline_properties_with_snapshot() {
    let registry = contract_registry();
    let failure_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.owner-task.v1
:END:
** Goal
"#;
    let success_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.owner-task.v1
:OWNER: alice
:END:
** Goal
"#;
    let failure_report = lint_org_with_options(
        failure_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let success_report = lint_org_with_options(
        success_source,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "failure clean: {}\n{}\nsuccess clean: {}\n{}",
        failure_report.is_clean(),
        failure_report.to_text("contract-owner-failure.org"),
        success_report.is_clean(),
        success_report.to_text("contract-owner-success.org")
    ));
}

#[test]
fn lint_contract_org_contract_block_queries_direct_child_with_snapshot() {
    let registry = contract_registry();
    let nested_goal_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.org-contract-task.v1
:END:
** Context
*** Goal
"#;
    let direct_goal_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.org-contract-task.v1
:END:
** Goal
"#;
    let nested_report = lint_org_with_options(
        nested_goal_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let direct_report = lint_org_with_options(
        direct_goal_source,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "nested clean: {}\n{}\ndirect clean: {}\n{}",
        nested_report.is_clean(),
        nested_report.to_text("contract-org-contract-nested.org"),
        direct_report.is_clean(),
        direct_report.to_text("contract-org-contract-direct.org")
    ));
}

#[test]
fn lint_contract_org_contract_block_supports_let_descendant_query_with_snapshot() {
    let registry = contract_registry();
    let sibling_link_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.evidence-link-task.v1
:END:
** Evidence
No link here.
** Context
[[https://example.test][outside]]
"#;
    let evidence_link_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.evidence-link-task.v1
:END:
** Evidence
[[https://example.test][inside]]
"#;
    let sibling_report = lint_org_with_options(
        sibling_link_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let evidence_report = lint_org_with_options(
        evidence_link_source,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "sibling clean: {}\n{}\nevidence clean: {}\n{}",
        sibling_report.is_clean(),
        sibling_report.to_text("contract-evidence-link-sibling.org"),
        evidence_report.is_clean(),
        evidence_report.to_text("contract-evidence-link-inside.org")
    ));
}

#[test]
fn lint_cli_loads_org_contract_registry_file_with_snapshot() {
    let dir = test_dir("lint-contract-org");
    fs::write(dir.join("contract.org"), contract_source()).unwrap();
    fs::write(
        dir.join("notes.org"),
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: ./contract.org#agent.task.v1
:END:
** Context
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args([
            "lint",
            "--format",
            "text",
            "--org-contract-registry",
            "contract.org",
            "notes.org",
        ])
        .output()
        .unwrap();

    insta::assert_snapshot!(command_snapshot(output));
}

#[test]
fn lint_cli_builds_org_contract_registry_from_directory_inputs() {
    let dir = test_dir("lint-contract-directory-registry");
    let contracts_dir = dir.join("contracts");
    fs::create_dir_all(&contracts_dir).unwrap();
    fs::write(contracts_dir.join("contract.org"), contract_source()).unwrap();
    fs::write(
        contracts_dir.join("notes.org"),
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: [[./contract.org][agent.task.v1]]
:END:
** Context
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_orgize"))
        .current_dir(&dir)
        .args(["lint", "--format", "text", "contracts"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("Task `Task A` must contain a Goal section."),
        "{stdout}\n{stderr}"
    );
    assert!(
        !stdout.contains("was not found in the loaded Org contract registry"),
        "{stdout}\n{stderr}"
    );
}

#[test]
fn lint_accepts_org_elements_selector_contract_query_with_snapshot() {
    let document = Org::parse(selector_contract_source()).document();
    let registry = parse_contracts_from_document(&document, None);
    let report = lint_org_with_options(
        r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.python-task.v1
:END:

#+BEGIN_SRC python
print("ready")
#+END_SRC
"#,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "clean: {}\n{}",
        report.is_clean(),
        report.to_text("selector-contract.org")
    ));
}

#[test]
fn lint_contract_org_contract_block_supports_summary_and_affiliated_conditions_with_snapshot() {
    let document = Org::parse(summary_condition_contract_source()).document();
    let registry = parse_contracts_from_document(&document, None);
    let unnamed_python_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.rich-task.v1
:END:

#+BEGIN_SRC python
print("ready")
#+END_SRC
%%(org-anniversary 1956 5 14)
"#;
    let missing_diary_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.rich-task.v1
:END:

#+NAME: task_runner
#+BEGIN_SRC python
print("ready")
#+END_SRC
"#;
    let mixed_language_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.rich-task.v1
:END:

#+NAME: task_runner
#+BEGIN_SRC python
print("ready")
#+END_SRC
#+BEGIN_SRC shell
echo ready
#+END_SRC
%%(org-anniversary 1956 5 14)
"#;
    let success_source = r#"* Task A
:PROPERTIES:
:CONTRACT_ORG: agent.rich-task.v1
:END:

#+NAME: task_runner
#+BEGIN_SRC python
print("ready")
#+END_SRC
%%(org-anniversary 1956 5 14)
"#;
    let unnamed_python_report = lint_org_with_options(
        unnamed_python_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let missing_diary_report = lint_org_with_options(
        missing_diary_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let mixed_language_report = lint_org_with_options(
        mixed_language_source,
        &LintOptions {
            org_contract_registry: registry.clone(),
            ..LintOptions::default()
        },
    );
    let success_report = lint_org_with_options(
        success_source,
        &LintOptions {
            org_contract_registry: registry,
            ..LintOptions::default()
        },
    );

    insta::assert_snapshot!(format!(
        "unnamed python clean: {}\n{}\n\
         missing diary clean: {}\n{}\n\
         mixed language clean: {}\n{}\n\
         success clean: {}\n{}",
        unnamed_python_report.is_clean(),
        unnamed_python_report.to_text("summary-affiliated-contract-unnamed-python.org"),
        missing_diary_report.is_clean(),
        missing_diary_report.to_text("summary-affiliated-contract-missing-diary.org"),
        mixed_language_report.is_clean(),
        mixed_language_report.to_text("summary-affiliated-contract-mixed-language.org"),
        success_report.is_clean(),
        success_report.to_text("summary-affiliated-contract-success.org")
    ));
}

fn contract_registry() -> orgize::ast::OrgContractRegistry {
    let document = Org::parse(contract_source()).document();
    parse_contracts_from_document(&document, None)
}

fn contract_source() -> &'static str {
    r#"* agent-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** must-have-goal-section
:PROPERTIES:
:ASSERT_ID: task.has-goal
:SEVERITY: error
:END:

#+BEGIN_SRC org-elements-query
(headline :descendant-of $scope :summary (title "Goal"))
#+END_SRC

#+BEGIN_SRC org-elements-expect
count >= 1
#+END_SRC

#+BEGIN_SRC jinja2 :name message
Task `{{ scope.title }}` must contain a Goal section.
#+END_SRC

* research-note-v1
:PROPERTIES:
:CONTRACT_ID: agent.research.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** must-have-evidence-section
:PROPERTIES:
:ASSERT_ID: research.has-evidence
:SEVERITY: error
:END:

#+BEGIN_SRC org-elements-query
(headline :descendant-of $scope :summary (title "Evidence"))
#+END_SRC

#+BEGIN_SRC org-elements-expect
count >= 1
#+END_SRC

#+BEGIN_SRC jinja2 :name message
Research note `{{ scope.title }}` must contain an Evidence section.
#+END_SRC

* owner-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.owner-task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** must-have-owner-property
:PROPERTIES:
:ASSERT_ID: task.has-owner
:SEVERITY: error
:END:

#+BEGIN_SRC org-elements-query
(headline :descendant-of $scope :property (:OWNER "alice"))
#+END_SRC

#+BEGIN_SRC org-elements-expect
exists
#+END_SRC

#+BEGIN_SRC jinja2 :name message
Task `{{ scope.title }}` must have OWNER `alice`.
#+END_SRC

* org-contract-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.org-contract-task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** must-have-direct-goal-child
:PROPERTIES:
:ASSERT_ID: task.has-direct-goal-child
:SEVERITY: error
:END:

#+BEGIN_SRC org-contract
(assert exists
  (headline :child-of $scope :property (:raw-value "Goal")))
#+END_SRC

#+BEGIN_SRC jinja2 :name message
Task `{{ scope.title }}` must contain a direct Goal child.
#+END_SRC

#+BEGIN_SRC jinja2 :name fix
Insert a direct Goal child heading.
#+END_SRC

* evidence-link-task-v1
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

fn selector_contract_source() -> &'static str {
    r#"* python-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.python-task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** has-python-block
:PROPERTIES:
:ASSERT_ID: task.has-python
:SEVERITY: warning
:END:

#+BEGIN_SRC org-elements-selector
(:org-element (:type src-block :language python))
#+END_SRC

#+BEGIN_SRC org-elements-expect
exists
#+END_SRC
"#
}

fn summary_condition_contract_source() -> &'static str {
    r#"* rich-task-v1
:PROPERTIES:
:CONTRACT_ID: agent.rich-task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:

** has-named-python-block
:PROPERTIES:
:ASSERT_ID: task.has-named-python-block
:SEVERITY: warning
:END:

#+BEGIN_SRC org-contract
(assert exists
  (and
    (src-block :name "task_runner")
    (or
      (= (summary language) "python")
      (= (summary language) "rust"))))
#+END_SRC

** has-diary-sexp
:PROPERTIES:
:ASSERT_ID: task.has-diary-sexp
:SEVERITY: warning
:END:

#+BEGIN_SRC org-contract
(assert count >= 1
  (diary-sexp :summary-contains (raw "org-anniversary")))
#+END_SRC

** has-no-non-python-block
:PROPERTIES:
:ASSERT_ID: task.has-no-non-python-block
:SEVERITY: warning
:END:

#+BEGIN_SRC org-contract
(assert not-exists
  (and
    (src-block)
    (not (= (summary language) "python"))))
#+END_SRC
"#
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orgize-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn command_snapshot(output: std::process::Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status.code().unwrap_or_default(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap()
    )
}
