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
