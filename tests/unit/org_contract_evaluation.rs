use crate::{
    Org,
    ast::{
        OrgContractAssertionStatus, OrgContractEvaluationContext, OrgContractEvaluationScope,
        OrgContractSeverity, evaluate_org_contract_with_context, parse_contract_reference,
        parse_contracts_from_document,
    },
};
use rowan::TextRange;
use std::path::Path;

#[test]
fn document_predicates_filter_contract_assertions_by_source_path() {
    let contract_source = r#"
* Skill filename contract
:PROPERTIES:
:CONTRACT_ID: skill.filename.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Skill template path
:PROPERTIES:
:ASSERT_ID: skill.template.path
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (org-data
    :path-contains "languages/org/templates/"
    :filename-suffix "_SKILL.org"
    :filename-stem-uppercase t))
#+END_SRC
"#;
    let contract_document = Org::parse(contract_source).document();
    let registry = parse_contracts_from_document(&contract_document, None);
    let contract = registry.contracts.first().expect("contract parsed");
    let document = Org::parse("* ASP Org\n").document();

    let matching_context =
        OrgContractEvaluationContext::with_source_path("languages/org/templates/ASP_ORG_SKILL.org");
    let matching = evaluate_org_contract_with_context(
        &document,
        contract,
        OrgContractEvaluationScope::document(),
        &matching_context,
    );
    assert_eq!(
        matching.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );

    let lowercase_context =
        OrgContractEvaluationContext::with_source_path("languages/org/templates/asp_org_skill.org");
    let lowercase = evaluate_org_contract_with_context(
        &document,
        contract,
        OrgContractEvaluationScope::document(),
        &lowercase_context,
    );
    assert_eq!(
        lowercase.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(lowercase.assertions[0].actual_count, 0);
}

#[test]
fn native_document_dir_property_limits_contract_evaluation_by_source_path() {
    let contract = parse_single_contract(
        r#"
* DIR contract
:PROPERTIES:
:CONTRACT_ID: dir.scope.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Has heading
:PROPERTIES:
:ASSERT_ID: dir.has-heading
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert count >= 1
  (headline))
#+END_SRC
"#,
    );
    let document = Org::parse(
        r#"#+PROPERTY: DIR /workspace/project/
* In scope
"#,
    )
    .document();

    let matching_context =
        OrgContractEvaluationContext::with_source_path("/workspace/project/README.org");
    let matching = evaluate_org_contract_with_context(
        &document,
        &contract,
        OrgContractEvaluationScope::document(),
        &matching_context,
    );
    assert_eq!(
        matching.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );

    let outside_context =
        OrgContractEvaluationContext::with_source_path("/workspace/other/README.org");
    let outside = evaluate_org_contract_with_context(
        &document,
        &contract,
        OrgContractEvaluationScope::document(),
        &outside_context,
    );
    assert_eq!(
        outside.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(outside.assertions[0].actual_count, 0);
}

#[test]
fn native_section_dir_property_overrides_inherited_document_dir_scope() {
    let contract = parse_single_contract(
        r#"
* DIR contract
:PROPERTIES:
:CONTRACT_ID: dir.scope.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:
** Has scoped heading
:PROPERTIES:
:ASSERT_ID: dir.has-scoped-heading
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert count >= 1
  (headline :at $scope))
#+END_SRC
"#,
    );
    let document = Org::parse(
        r#"#+PROPERTY: DIR /workspace/project/
* Major Project Alpha
:PROPERTIES:
:DIR: /workspace/alpha/
:END:
** Child
"#,
    )
    .document();
    let scope = OrgContractEvaluationScope::section(
        "Major Project Alpha",
        vec!["Major Project Alpha".to_string()],
        TextRange::new(0.into(), 0.into()),
    );

    let matching_context =
        OrgContractEvaluationContext::with_source_path("/workspace/alpha/README.org");
    let matching =
        evaluate_org_contract_with_context(&document, &contract, scope.clone(), &matching_context);
    assert_eq!(
        matching.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );

    let inherited_document_context =
        OrgContractEvaluationContext::with_source_path("/workspace/project/README.org");
    let outside = evaluate_org_contract_with_context(
        &document,
        &contract,
        scope,
        &inherited_document_context,
    );
    assert_eq!(
        outside.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(outside.assertions[0].actual_count, 0);
}

#[test]
fn native_dir_property_value_expands_environment_variables_and_org_macros() {
    let contract = parse_single_contract(
        r#"
* DIR contract
:PROPERTIES:
:CONTRACT_ID: dir.scope.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Has heading
:PROPERTIES:
:ASSERT_ID: dir.has-heading
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert count >= 1
  (headline))
#+END_SRC
"#,
    );
    let (home_var, home) = std::env::var("HOME")
        .map(|home| ("HOME", home))
        .or_else(|_| std::env::var("USERPROFILE").map(|home| ("USERPROFILE", home)))
        .expect("HOME or USERPROFILE available for env expansion test");
    let env_source = format!("#+PROPERTY: DIR ${{{home_var}}}/asp-dir-scope/\n* Env scope\n");
    let env_document = Org::parse(&env_source).document();
    let env_context =
        OrgContractEvaluationContext::with_source_path(format!("{home}/asp-dir-scope/README.org"));
    let env_result = evaluate_org_contract_with_context(
        &env_document,
        &contract,
        OrgContractEvaluationScope::document(),
        &env_context,
    );
    assert_eq!(
        env_result.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );

    let macro_document = Org::parse(
        r#"#+MACRO: project-root /workspace/project
#+PROPERTY: DIR {{{project-root}}}/alpha/
* Macro scope
"#,
    )
    .document();
    let macro_context =
        OrgContractEvaluationContext::with_source_path("/workspace/project/alpha/README.org");
    let macro_result = evaluate_org_contract_with_context(
        &macro_document,
        &contract,
        OrgContractEvaluationScope::document(),
        &macro_context,
    );
    assert_eq!(
        macro_result.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
}

#[test]
fn native_dir_property_value_expands_command_substitution() {
    let contract = parse_single_contract(
        r#"
* DIR contract
:PROPERTIES:
:CONTRACT_ID: dir.scope.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Has heading
:PROPERTIES:
:ASSERT_ID: dir.has-heading
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert count >= 1
  (headline))
#+END_SRC
"#,
    );
    let document = Org::parse(
        r#"#+PROPERTY: DIR $(printf /workspace/generated)
* Command scope
"#,
    )
    .document();
    let context = OrgContractEvaluationContext::with_source_path("/workspace/generated/README.org");
    let result = evaluate_org_contract_with_context(
        &document,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    assert_eq!(
        result.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
}

#[test]
fn contract_reference_paths_match_windows_style_relative_org_links() {
    let contract_document = Org::parse(
        r#"
* Contract
:PROPERTIES:
:CONTRACT_ID: agent.evidence-link-task.v1
:CONTRACT_SCOPE: subtree
:CONTRACT_KIND: org-elements
:END:
** Evidence
:PROPERTIES:
:ASSERT_ID: evidence.required
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (link :scheme "https"))
#+END_SRC
"#,
    )
    .document();
    let registry = parse_contracts_from_document(
        &contract_document,
        Some(Path::new("contracts/contract.org")),
    );
    let reference =
        parse_contract_reference(r"[[..\contracts\contract.org][agent.evidence-link-task.v1]]")
            .with_source_relative_path(Some(Path::new("templates/skill.org")));

    assert!(registry.resolve(&reference).is_some());
}

#[test]
fn query_level_or_matches_node_property_branches() {
    let contract = parse_single_contract(
        r#"
* Plan lifecycle state contract
:PROPERTIES:
:CONTRACT_ID: plan.lifecycle-state.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Status is active or complete
:PROPERTIES:
:ASSERT_ID: plan.status-is-lifecycle-state
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (or
    (node-property :summary (key "STATUS") :summary (value "active"))
    (node-property :summary (key "STATUS") :summary (value "complete"))))
#+END_SRC
"#,
    );
    let document = Org::parse(
        r#"
* TODO Plan
:PROPERTIES:
:STATUS: active
:END:
"#,
    )
    .document();
    let evaluation = evaluate_org_contract_with_context(
        &document,
        &contract,
        OrgContractEvaluationScope::document(),
        &OrgContractEvaluationContext::with_source_path("plans/agent-plan-example.org"),
    );

    assert_eq!(evaluation.assertions.len(), 1);
    assert_eq!(
        evaluation.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
    assert_eq!(evaluation.assertions[0].actual_count, 1);
}

#[test]
fn contract_kind_sugar_can_restrict_properties_to_document_root() {
    let contract = parse_single_contract(
        r#"
* Root source kind contract
:PROPERTIES:
:CONTRACT_ID: source.root-kind.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Source kind is primary
:PROPERTIES:
:ASSERT_ID: source.has-root-kind
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (node-property
    :outline-depth 0
    :summary (key "SOURCE_KIND")
    :summary (value "PRIMARY")))
#+END_SRC
"#,
    );
    let document = Org::parse(
        r#"
:PROPERTIES:
:SOURCE_KIND: INTERPRETATION
:END:
* Masking child
:PROPERTIES:
:SOURCE_KIND: PRIMARY
:END:
"#,
    )
    .document();
    let evaluation = evaluate_org_contract_with_context(
        &document,
        &contract,
        OrgContractEvaluationScope::document(),
        &OrgContractEvaluationContext::with_source_path("sources/example.org"),
    );

    assert_eq!(evaluation.assertions.len(), 1);
    assert_eq!(
        evaluation.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(evaluation.assertions[0].actual_count, 0);
}

#[test]
fn table_column_nonempty_does_not_shift_across_an_empty_cell() {
    let contract = parse_single_contract(
        r#"
* Engineering column contract
:PROPERTIES:
:CONTRACT_ID: engineering.column.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Engineering property is substantive
:PROPERTIES:
:ASSERT_ID: engineering.has-property
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (table-cell :column "Engineering property" :header false :nonempty true))
#+END_SRC
"#,
    );
    let document = Org::parse(
        r#"
| Principle | Engineering property | Owner |
|-----------+----------------------+-------|
| P-001     |                      | ASP   |
"#,
    )
    .document();
    let evaluation = evaluate_org_contract_with_context(
        &document,
        &contract,
        OrgContractEvaluationScope::document(),
        &OrgContractEvaluationContext::default(),
    );

    assert_eq!(evaluation.assertions.len(), 1);
    assert_eq!(
        evaluation.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(evaluation.assertions[0].actual_count, 0);
}

#[test]
fn predicate_or_groups_inside_and_are_intersected() {
    let contract = parse_single_contract(
        r#"
* Principle classification contract
:PROPERTIES:
:CONTRACT_ID: principle.classification.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Status and kind enums
:PROPERTIES:
:ASSERT_ID: principle.has-valid-status-and-kind
:SEVERITY: error
:END:
#+BEGIN_SRC org-contract
(assert exists
  (and
    (headline)
    (or
      (= (property "STATUS") "draft")
      (= (property "STATUS") "accepted"))
    (or
      (= (property "KIND") "foundational")
      (= (property "KIND") "refinement"))))
#+END_SRC
"#,
    );
    for (kind, expected) in [
        ("foundational", OrgContractAssertionStatus::Passed),
        ("unclassified", OrgContractAssertionStatus::Failed),
    ] {
        let document = Org::parse(format!(
            "* Principle\n:PROPERTIES:\n:STATUS: draft\n:KIND: {kind}\n:END:\n"
        ))
        .document();
        let evaluation = evaluate_org_contract_with_context(
            &document,
            &contract,
            OrgContractEvaluationScope::document(),
            &OrgContractEvaluationContext::with_source_path("principles/test.org"),
        );
        assert_eq!(evaluation.assertions[0].status, expected, "kind={kind}");
    }
}

#[test]
fn named_org_contract_blocks_define_assertions_without_heading_properties() {
    let contract_source = r#"
* Evidence link contract
:PROPERTIES:
:CONTRACT_ID: task.evidence-link.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:

#+NAME: task.evidence-has-link
#+BEGIN_SRC org-contract :severity warning
(assert count >= 1
  (headline :summary (title "Task")))
#+END_SRC

#+NAME: task.evidence-has-link.message
#+BEGIN_SRC jinja2
Task must include a replayable evidence link.
#+END_SRC
"#;
    let contract_document = Org::parse(contract_source).document();
    let source_blocks = contract_document.source_block_records();
    assert_eq!(
        source_blocks[0].name.as_deref(),
        Some("task.evidence-has-link")
    );

    let registry = parse_contracts_from_document(&contract_document, None);
    let contract = registry.contracts.first().expect("contract parsed");
    assert_eq!(contract.assertions.len(), 1);
    assert_eq!(contract.assertions[0].id, "task.evidence-has-link");
    assert_eq!(
        contract.assertions[0].severity,
        OrgContractSeverity::Warning
    );
    assert_eq!(
        contract.assertions[0].message.as_deref().map(str::trim),
        Some("Task must include a replayable evidence link.")
    );

    let document = Org::parse(
        r#"
* Task
[[https://example.test][evidence]]
"#,
    )
    .document();
    let evaluation = evaluate_org_contract_with_context(
        &document,
        contract,
        OrgContractEvaluationScope::document(),
        &OrgContractEvaluationContext::with_source_path("notes.org"),
    );

    assert_eq!(evaluation.assertions.len(), 1);
    assert_eq!(
        evaluation.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
    assert_eq!(evaluation.assertions[0].actual_count, 1);
}

#[test]
fn custom_document_keywords_are_queryable_case_insensitively() {
    let contract = parse_single_contract(
        r#"
* Document status contract
:PROPERTIES:
:CONTRACT_ID: document.status
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Status keyword
:PROPERTIES:
:ASSERT_ID: document.status.keyword
:SEVERITY: error
:END:
#+BEGIN_SRC org-elements-selector
(:org-element (:type keyword :name status))
#+END_SRC
"#,
    );
    assert_eq!(contract.assertions.len(), 1, "contract assertion parsed");
    let context = OrgContractEvaluationContext::default();

    let matching_document = Org::parse("#+STATUS: active\n").document();
    let matching = evaluate_org_contract_with_context(
        &matching_document,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    assert_eq!(
        matching.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );

    let missing_document = Org::parse("#+TITLE: No status\n").document();
    let missing = evaluate_org_contract_with_context(
        &missing_document,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    assert_eq!(
        missing.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(missing.assertions[0].actual_count, 0);
}

fn parse_single_contract(source: &str) -> crate::ast::OrgContract {
    let contract_document = Org::parse(source).document();
    let registry = parse_contracts_from_document(&contract_document, None);
    registry
        .contracts
        .into_iter()
        .next()
        .expect("contract parsed")
}

#[test]
fn contract_count_can_match_a_binding_to_require_complete_nodes() {
    let contract = parse_single_contract(
        r#"
* Principle nodes
:PROPERTIES:
:CONTRACT_ID: principle.nodes.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Complete metadata on every principle
:PROPERTIES:
:ASSERT_ID: principle.nodes.complete
:SEVERITY: error
:END:
#+begin_src org-contract
(let ((principles
       (headline :property-contains ("PRINCIPLE_ID" ""))))
  (assert count == $principles
    (and
      (headline
        :property-contains ("PRINCIPLE_ID" "")
        :property-contains ("PRINCIPLE_STATUS" "")
        :property-contains ("REVISION" ""))
      (not (= (property "PRINCIPLE_ID") ""))
      (not (= (property "PRINCIPLE_STATUS") ""))
      (not (= (property "REVISION") "")))))
#+end_src
"#,
    );
    let complete = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:PRINCIPLE_STATUS: proposed\n:REVISION: 1\n:END:\n* B\n:PROPERTIES:\n:PRINCIPLE_ID: P-002\n:PRINCIPLE_STATUS: draft\n:REVISION: 1\n:END:\n",
    )
    .document();
    let incomplete = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:PRINCIPLE_STATUS: proposed\n:REVISION: 1\n:END:\n* B\n:PROPERTIES:\n:PRINCIPLE_ID: P-002\n:PRINCIPLE_STATUS: draft\n:END:\n",
    )
    .document();
    let context = OrgContractEvaluationContext::default();
    let complete = evaluate_org_contract_with_context(
        &complete,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    let incomplete = evaluate_org_contract_with_context(
        &incomplete,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    assert_eq!(
        complete.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
    assert_eq!(
        incomplete.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
}

#[test]
fn contract_value_sets_can_require_exact_trace_coverage() {
    let contract = parse_single_contract(
        r#"
* Trace coverage
:PROPERTIES:
:CONTRACT_ID: trace.coverage.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Every principle is traced
:PROPERTIES:
:ASSERT_ID: trace.covers-principles
:SEVERITY: error
:END:
#+begin_src org-contract
(let ((principles
       (headline :property-contains ("PRINCIPLE_ID" ""))))
  (assert value-set ==
    (source $principles (property "PRINCIPLE_ID"))
    (target
      (table-cell :column "Principle ID" :header false :nonempty true)
      (summary "text"))))
#+end_src
"#,
    );
    let complete = Org::parse(
        r#"* A
:PROPERTIES:
:PRINCIPLE_ID: P-001
:END:
* B
:PROPERTIES:
:PRINCIPLE_ID: P-002
:END:
| Principle ID | Evidence |
|--------------+----------|
| P-001        | one      |
| P-002        | two      |
"#,
    )
    .document();
    let missing = Org::parse(
        r#"* A
:PROPERTIES:
:PRINCIPLE_ID: P-001
:END:
* B
:PROPERTIES:
:PRINCIPLE_ID: P-002
:END:
| Principle ID | Evidence |
|--------------+----------|
| P-001        | one      |
| P-001        | duplicate |
"#,
    )
    .document();

    let context = OrgContractEvaluationContext::default();
    let complete_evaluation = evaluate_org_contract_with_context(
        &complete,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    let missing_evaluation = evaluate_org_contract_with_context(
        &missing,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    assert_eq!(
        complete_evaluation.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
    assert_eq!(
        missing_evaluation.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
}

#[test]
fn contract_table_rows_can_require_named_columns_on_the_same_row() {
    let contract = parse_single_contract(
        r#"
* Trace rows
:PROPERTIES:
:CONTRACT_ID: trace.rows.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Every trace row is complete
:PROPERTIES:
:ASSERT_ID: trace.rows.complete
:SEVERITY: error
:END:
#+begin_src org-contract
(let ((principles
       (headline :property-contains ("PRINCIPLE_ID" ""))))
  (assert count == $principles
    (table-row
      :header false
      :column-nonempty "Principle ID"
      :column-nonempty "Engineering direction"
      :column-nonempty "Downstream owner")))
#+end_src
"#,
    );
    let complete = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:END:\n| Principle ID | Engineering direction | Downstream owner |\n|--------------+-----------------------+------------------|\n| P-001        | Query from AST        | Orgize           |\n",
    )
    .document();
    let compensated = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:END:\n| Principle ID | Engineering direction | Downstream owner |\n|--------------+-----------------------+------------------|\n| P-001        |                       | Orgize           |\n|              | Query from AST        | Orgize           |\n",
    )
    .document();
    let headerless = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:END:\n| Principle ID | Engineering direction | Downstream owner |\n| P-001        | Query from AST        | Orgize           |\n",
    )
    .document();
    let multirow_header = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:END:\n| Principle ID | Engineering direction | Downstream owner |\n| P-001        | Header explanation    | Header owner     |\n|--------------+------------------------+------------------|\n| P-001        |                        | Orgize           |\n",
    )
    .document();
    let duplicate_header = Org::parse(
        "* A\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:END:\n| Principle ID | Principle ID | Engineering direction | Downstream owner |\n|--------------+--------------+-----------------------+------------------|\n|              | P-001        | Query from AST        | Orgize           |\n",
    )
    .document();
    let context = OrgContractEvaluationContext::default();
    let complete = evaluate_org_contract_with_context(
        &complete,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    let compensated = evaluate_org_contract_with_context(
        &compensated,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    let headerless = evaluate_org_contract_with_context(
        &headerless,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    let multirow_header = evaluate_org_contract_with_context(
        &multirow_header,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    let duplicate_header = evaluate_org_contract_with_context(
        &duplicate_header,
        &contract,
        OrgContractEvaluationScope::document(),
        &context,
    );
    assert_eq!(
        complete.assertions[0].status,
        OrgContractAssertionStatus::Passed
    );
    assert_eq!(
        compensated.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(
        headerless.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(
        multirow_header.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
    assert_eq!(
        duplicate_header.assertions[0].status,
        OrgContractAssertionStatus::Failed
    );
}

#[test]
fn contract_positive_integer_predicate_rejects_noncanonical_and_nonpositive_values() {
    let contract = parse_single_contract(
        r#"
* Revision contract
:PROPERTIES:
:CONTRACT_ID: revision.positive-integer.v1
:CONTRACT_SCOPE: document
:CONTRACT_KIND: org-elements
:END:
** Positive revision
:PROPERTIES:
:ASSERT_ID: revision.is-positive-integer
:SEVERITY: error
:END:
#+begin_src org-contract
(assert count == 1
  (and
    (headline :property-contains ("PRINCIPLE_ID" ""))
    (positive-integer (property "REVISION"))))
#+end_src
"#,
    );

    for (revision, expected) in [
        ("1", OrgContractAssertionStatus::Passed),
        ("42", OrgContractAssertionStatus::Passed),
        ("0", OrgContractAssertionStatus::Failed),
        ("01", OrgContractAssertionStatus::Failed),
        ("-1", OrgContractAssertionStatus::Failed),
        ("latest", OrgContractAssertionStatus::Failed),
        ("+1", OrgContractAssertionStatus::Failed),
    ] {
        let document = Org::parse(format!(
            "* Principle\n:PROPERTIES:\n:PRINCIPLE_ID: P-001\n:REVISION: {revision}\n:END:\n"
        ))
        .document();
        let evaluation = evaluate_org_contract_with_context(
            &document,
            &contract,
            OrgContractEvaluationScope::document(),
            &OrgContractEvaluationContext::default(),
        );
        assert_eq!(
            evaluation.assertions[0].status, expected,
            "unexpected status for revision {revision:?}"
        );
    }
}
