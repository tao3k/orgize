//! Contract evaluation facts for `CONTRACT_ORG`.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    OrgContract, OrgContractAssertion, OrgContractAssertionEvaluation, OrgContractAssertionStatus,
    OrgContractEvaluation, OrgContractEvaluationScope, OrgContractQuery, OrgContractRelativeScope,
    OrgElementGraph, OrgElementId, OrgElementsIndexQuery, ParsedAnnotation, ParsedAst,
};

/// Evaluates a resolved Org contract over a source-backed document scope.
pub fn evaluate_org_contract(
    document: &ParsedAst,
    contract: &OrgContract,
    scope: OrgContractEvaluationScope,
) -> OrgContractEvaluation {
    let graph = document.org_elements_graph();
    let assertions = contract
        .assertions
        .iter()
        .map(|assertion| evaluate_assertion(&graph, assertion, &scope))
        .collect();
    OrgContractEvaluation {
        contract_id: contract.id.clone(),
        scope,
        assertions,
    }
}

fn evaluate_assertion(
    graph: &OrgElementGraph<ParsedAnnotation>,
    assertion: &OrgContractAssertion,
    scope: &OrgContractEvaluationScope,
) -> OrgContractAssertionEvaluation {
    let mut binding_sets = BTreeMap::<String, BTreeSet<OrgElementId>>::new();
    for binding in &assertion.bindings {
        let query = scoped_contract_query(&binding.query, scope);
        binding_sets.insert(
            binding.name.clone(),
            query_graph_ids(graph, &query, &binding_sets),
        );
    }
    let query = scoped_contract_query(&assertion.query, scope);
    let matched = query_graph_ids(graph, &query, &binding_sets);
    let actual_count = matched.len();
    let bindings = binding_sets
        .into_iter()
        .map(|(name, ids)| (name, ids.into_iter().collect()))
        .collect();
    OrgContractAssertionEvaluation {
        assertion_id: assertion.id.clone(),
        severity: assertion.severity,
        expectation: assertion.expectation.clone(),
        actual_count,
        status: if assertion.expectation.check(actual_count) {
            OrgContractAssertionStatus::Passed
        } else {
            OrgContractAssertionStatus::Failed
        },
        matched_ids: matched.into_iter().collect(),
        bindings,
        message_template: assertion.message.clone(),
        fix_template: assertion.fix.clone(),
    }
}

fn scoped_contract_query(
    query: &OrgContractQuery,
    scope: &OrgContractEvaluationScope,
) -> OrgContractQuery {
    match scope {
        OrgContractEvaluationScope::Document { .. } => query.clone(),
        OrgContractEvaluationScope::Section { outline_path, .. } => query
            .clone()
            .apply_subtree_scope_prefix(outline_path.clone()),
    }
}

fn query_graph_ids(
    graph: &OrgElementGraph<ParsedAnnotation>,
    query: &OrgContractQuery,
    bindings: &BTreeMap<String, BTreeSet<OrgElementId>>,
) -> BTreeSet<OrgElementId> {
    let Some(index_query) = index_query_with_relative_scope(query, bindings) else {
        return BTreeSet::new();
    };
    graph
        .query(&index_query)
        .iter()
        .map(|record| record.id)
        .collect()
}

fn index_query_with_relative_scope(
    query: &OrgContractQuery,
    bindings: &BTreeMap<String, BTreeSet<OrgElementId>>,
) -> Option<OrgElementsIndexQuery> {
    let index_query = query.to_index_query();
    match &query.relative_to {
        None => Some(index_query),
        Some(OrgContractRelativeScope::DescendantOfBinding(binding)) => {
            let roots = bindings.get(binding)?;
            (!roots.is_empty()).then(|| index_query.descendant_of_any(roots.iter().copied()))
        }
        Some(OrgContractRelativeScope::ChildOfBinding(binding)) => {
            let roots = bindings.get(binding)?;
            (!roots.is_empty()).then(|| index_query.child_of_any(roots.iter().copied()))
        }
        Some(OrgContractRelativeScope::AtBinding(binding)) => {
            let roots = bindings.get(binding)?;
            (!roots.is_empty()).then(|| index_query.at_any(roots.iter().copied()))
        }
    }
}
