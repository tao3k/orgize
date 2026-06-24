use crate::{
    Org,
    ast::{
        OrgContractAssertionStatus, OrgContractEvaluationContext, OrgContractEvaluationScope,
        evaluate_org_contract_with_context, parse_contracts_from_document,
    },
};
use rowan::TextRange;

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
    let home = std::env::var("HOME").expect("HOME available for env expansion test");
    let env_document = Org::parse(
        r#"#+PROPERTY: DIR ${HOME}/asp-dir-scope/
* Env scope
"#,
    )
    .document();
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

fn parse_single_contract(source: &str) -> crate::ast::OrgContract {
    let contract_document = Org::parse(source).document();
    let registry = parse_contracts_from_document(&contract_document, None);
    registry
        .contracts
        .into_iter()
        .next()
        .expect("contract parsed")
}
