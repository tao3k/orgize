//! Execution of Scheme-AOT Org Contract plans over the generated Element graph.
//!
//! This layer knows no Org syntax or S-expression language. The Scheme module
//! admits every kind, field, relation, and expectation before code generation.

use std::collections::HashMap;

use gerbil_parser_rowan::{GraphIndexError, GraphProjectionSpec, GraphRecord, GraphRelation};

use crate::{
    org_aot::OrgAotDocument,
    org_element_query::{OrgElementPropertyRule, element_property_matches},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Relationship between a selected Element and the query target.
pub enum ContractRelation {
    Any,
    At,
    ChildOf,
    DescendantOf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Cardinality test applied to a selected Element set.
pub enum ContractOperator {
    AtLeast,
    Exactly,
    AtMost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Diagnostic level emitted when an assertion fails.
pub enum ContractSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Org heading or document region that owns a contract.
pub enum ContractScope {
    Document,
    Subtree,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Immutable Rust projection of a Scheme Org Element query.
pub struct ContractQueryRule {
    pub node_kind: &'static str,
    pub groups: &'static [&'static [OrgElementPropertyRule]],
    pub relation: ContractRelation,
    pub target_scope: bool,
    pub target_binding: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Cardinality expectation projected from Scheme.
pub struct ContractExpectationRule {
    pub operator: ContractOperator,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Named intermediate Element selection for later assertions.
pub struct ContractBindingRule {
    pub name: &'static str,
    pub query: ContractQueryRule,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A Scheme-authored assertion projected into Rust data.
pub struct ContractAssertionRule {
    pub id: &'static str,
    pub severity: ContractSeverity,
    pub bindings: &'static [ContractBindingRule],
    pub query: ContractQueryRule,
    pub expectation: ContractExpectationRule,
    pub message: Option<&'static str>,
    pub fix: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A complete AOT contract bound to one Element projection digest.
pub struct ContractRule {
    pub id: &'static str,
    pub graph_digest: &'static str,
    pub scope: ContractScope,
    pub assertions: &'static [ContractAssertionRule],
}

/// Immutable group of contracts generated from one Org source document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractPack {
    pub rules: &'static [ContractRule],
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Result of evaluating one assertion over an Org Element graph.
pub struct ContractResult {
    pub assertion_id: &'static str,
    pub matched_count: usize,
    pub passed: bool,
    pub severity: ContractSeverity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Rejection reason for a generated plan or graph input.
pub enum ContractExecutionError {
    StaleGraph,
    InvalidGraph,
    InvalidScope,
    UnknownBinding,
    DuplicateBinding,
}

/// Identity of the Element node to which a contract is applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractScopeNodeId(pub usize);

fn target_ids(
    query: ContractQueryRule,
    scope_id: usize,
    bindings: &HashMap<&'static str, Vec<usize>>,
) -> Result<Vec<usize>, ContractExecutionError> {
    if query.target_scope {
        Ok(vec![scope_id])
    } else if let Some(name) = query.target_binding {
        bindings
            .get(name)
            .cloned()
            .ok_or(ContractExecutionError::UnknownBinding)
    } else {
        Ok(Vec::new())
    }
}

fn field_matches(
    document: &OrgAotDocument,
    record: &GraphRecord,
    query: ContractQueryRule,
) -> bool {
    query.groups.iter().any(|group| {
        group.iter().all(|property| {
            element_property_matches(
                document,
                record,
                property.name,
                property.value,
                property.matcher,
            )
        })
    })
}

fn select(
    query: ContractQueryRule,
    document: &OrgAotDocument,
    scope_id: usize,
    bindings: &HashMap<&'static str, Vec<usize>>,
) -> Result<Vec<usize>, ContractExecutionError> {
    let targets = target_ids(query, scope_id, bindings)?;
    let relation = match query.relation {
        ContractRelation::Any => GraphRelation::Any,
        ContractRelation::At => GraphRelation::At,
        ContractRelation::ChildOf => GraphRelation::ChildOf,
        ContractRelation::DescendantOf => GraphRelation::DescendantOf,
    };
    document
        .graph_index()
        .select(
            document.records(),
            scope_id,
            query.node_kind,
            relation,
            &targets,
            |record| field_matches(document, record, query),
        )
        .map_err(|error| match error {
            GraphIndexError::InvalidScope => ContractExecutionError::InvalidScope,
            GraphIndexError::InvalidRecord | GraphIndexError::InvalidTarget => {
                ContractExecutionError::InvalidGraph
            }
        })
}

/// Run an admitted, generated contract over the source-backed Element graph.
///
/// # Errors
///
/// Rejects malformed graph ancestry, scope IDs, and unresolved binding names.
pub fn evaluate_contract(
    contract: &ContractRule,
    graph_spec: &GraphProjectionSpec,
    document: &OrgAotDocument,
    scope: ContractScopeNodeId,
) -> Result<Vec<ContractResult>, ContractExecutionError> {
    if contract.graph_digest != graph_spec.projection_digest {
        return Err(ContractExecutionError::StaleGraph);
    }
    let records = document.records();
    let scope_id = scope.0;
    if scope_id >= records.len() {
        return Err(ContractExecutionError::InvalidScope);
    }
    match contract.scope {
        ContractScope::Document if scope_id != 0 || records[scope_id].kind != "org-data" => {
            return Err(ContractExecutionError::InvalidScope);
        }
        ContractScope::Subtree if records[scope_id].kind != "headline" => {
            return Err(ContractExecutionError::InvalidScope);
        }
        _ => {}
    }
    let mut results = Vec::with_capacity(contract.assertions.len());
    for assertion in contract.assertions {
        let mut bindings = HashMap::<&'static str, Vec<usize>>::new();
        for binding in assertion.bindings {
            if bindings.contains_key(binding.name) {
                return Err(ContractExecutionError::DuplicateBinding);
            }
            let selected = select(binding.query, document, scope_id, &bindings)?;
            bindings.insert(binding.name, selected);
        }
        let count = select(assertion.query, document, scope_id, &bindings)?.len();
        let passed = match assertion.expectation.operator {
            ContractOperator::AtLeast => count >= assertion.expectation.count,
            ContractOperator::Exactly => count == assertion.expectation.count,
            ContractOperator::AtMost => count <= assertion.expectation.count,
        };
        results.push(ContractResult {
            assertion_id: assertion.id,
            matched_count: count,
            passed,
            severity: assertion.severity,
        });
    }
    Ok(results)
}

#[cfg(test)]
#[path = "../tests/unit/contract_feature.rs"]
mod tests;
