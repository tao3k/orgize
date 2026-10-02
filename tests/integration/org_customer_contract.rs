//! Consumer Scheme Contract authoring to Cargo-only graph execution.

mod customer_contract_plan;

use orgize::{contract_feature::ContractScopeNodeId, org_aot::parse_org_aot};

#[test]
fn consumer_contract_pack_evaluates_custom_scope_and_todo_policy() {
    let document = parse_org_aot(
        "#+TODO: WAIT | DONE\n* WAIT Review\nSee [[https://example.test][spec]].\n** DONE Child\n* WAIT Audit\n",
    )
    .expect("Scheme-AOT Org parser accepts the consumer document");
    let review = document
        .records()
        .iter()
        .find(|record| record.kind == "headline" && record.field("title") == Some("WAIT Review"))
        .expect("review headline");
    let link_rule = customer_contract_plan::CONTRACTS
        .rules
        .iter()
        .find(|rule| rule.id == "customer.review-evidence")
        .expect("consumer subtree rule");
    let todo_rule = customer_contract_plan::CONTRACTS
        .rules
        .iter()
        .find(|rule| rule.id == "customer.has-task")
        .expect("consumer document rule");

    let link_result = document
        .evaluate_contract(link_rule, ContractScopeNodeId(review.id))
        .expect("evaluate consumer subtree rule");
    assert_eq!(link_result[0].assertion_id, "customer.review-has-link");
    assert_eq!(link_result[0].matched_count, 1, "{:#?}", document.records());
    assert!(link_result[0].passed);

    let todo_result = document
        .evaluate_contract(todo_rule, ContractScopeNodeId(0))
        .expect("evaluate consumer document rule");
    assert_eq!(todo_result[0].assertion_id, "customer.has-task-headline");
    assert_eq!(todo_result[0].matched_count, 2, "{:#?}", document.records());
    assert!(todo_result[0].passed);
}

#[test]
fn consumer_contract_pack_reports_missing_evidence_without_rust_parser() {
    let document = parse_org_aot("* Review\nNo linked evidence.\n")
        .expect("Scheme-AOT Org parser accepts the consumer document");
    let review = document
        .records()
        .iter()
        .find(|record| record.kind == "headline")
        .expect("review headline");
    let rule = &customer_contract_plan::CONTRACTS.rules[0];
    let result = document
        .evaluate_contract(rule, ContractScopeNodeId(review.id))
        .expect("evaluate consumer subtree rule");
    assert_eq!(result[0].matched_count, 0);
    assert!(!result[0].passed);
    assert_eq!(rule.assertions[0].message, Some("review requires a link"));
}
