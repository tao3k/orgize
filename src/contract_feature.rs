//! Execution of Scheme-AOT Org Contract plans over the generated Element graph.
//!
//! This layer knows no Org syntax or S-expression language. The Scheme module
//! admits every kind, field, relation, and expectation before code generation.

use std::collections::{HashMap, HashSet};

use gerbil_parser_rowan::{GraphProjectionSpec, GraphRecord};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Relationship between a selected Element and the query target.
pub enum ContractRelation {
    Any,
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
    pub field_name: Option<&'static str>,
    pub field_value: Option<&'static str>,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A complete AOT contract bound to one Element projection digest.
pub struct ContractRule {
    pub id: &'static str,
    pub graph_digest: &'static str,
    pub scope: ContractScope,
    pub assertions: &'static [ContractAssertionRule],
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

struct GraphIndex<'a> {
    records: &'a [GraphRecord],
    subtree_end: Vec<usize>,
}

impl<'a> GraphIndex<'a> {
    fn new(records: &'a [GraphRecord]) -> Result<Self, ContractExecutionError> {
        let mut subtree_end = vec![records.len(); records.len()];
        let mut stack = Vec::<usize>::new();
        for (position, record) in records.iter().enumerate() {
            if record.id != position {
                return Err(ContractExecutionError::InvalidGraph);
            }
            while stack.last().copied() != record.parent_id {
                let Some(closed) = stack.pop() else {
                    return Err(ContractExecutionError::InvalidGraph);
                };
                subtree_end[closed] = record.id;
            }
            stack.push(record.id);
        }
        Ok(Self {
            records,
            subtree_end,
        })
    }

    fn descendant_or_self(&self, ancestor: usize, node: usize) -> bool {
        ancestor <= node && node < self.subtree_end[ancestor]
    }

    fn descendant(&self, ancestor: usize, node: usize) -> bool {
        ancestor < node && node < self.subtree_end[ancestor]
    }
}

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

fn select(
    query: ContractQueryRule,
    graph: &GraphIndex<'_>,
    scope_id: usize,
    bindings: &HashMap<&'static str, Vec<usize>>,
) -> Result<Vec<usize>, ContractExecutionError> {
    let targets = target_ids(query, scope_id, bindings)?;
    let child_targets: HashSet<_> = targets.iter().copied().collect();
    let mut matches = Vec::new();
    for record in graph.records {
        if !graph.descendant_or_self(scope_id, record.id)
            || record.kind != query.node_kind
            || query
                .field_name
                .is_some_and(|field| record.field(field) != query.field_value)
        {
            continue;
        }
        let related = match query.relation {
            ContractRelation::Any => true,
            ContractRelation::ChildOf => record
                .parent_id
                .is_some_and(|parent| child_targets.contains(&parent)),
            ContractRelation::DescendantOf => targets
                .iter()
                .any(|&target| graph.descendant(target, record.id)),
        };
        if related {
            matches.push(record.id);
        }
    }
    Ok(matches)
}

/// Run an admitted, generated contract over the source-backed Element graph.
///
/// # Errors
///
/// Rejects malformed graph ancestry, scope IDs, and unresolved binding names.
pub fn evaluate_contract(
    contract: &ContractRule,
    graph_spec: &GraphProjectionSpec,
    records: &[GraphRecord],
    scope: ContractScopeNodeId,
) -> Result<Vec<ContractResult>, ContractExecutionError> {
    if contract.graph_digest != graph_spec.projection_digest {
        return Err(ContractExecutionError::StaleGraph);
    }
    let graph = GraphIndex::new(records)?;
    let scope_id = scope.0;
    if scope_id >= records.len() {
        return Err(ContractExecutionError::InvalidScope);
    }
    let mut results = Vec::with_capacity(contract.assertions.len());
    for assertion in contract.assertions {
        let mut bindings = HashMap::<&'static str, Vec<usize>>::new();
        for binding in assertion.bindings {
            if bindings.contains_key(binding.name) {
                return Err(ContractExecutionError::DuplicateBinding);
            }
            let selected = select(binding.query, &graph, scope_id, &bindings)?;
            bindings.insert(binding.name, selected);
        }
        let count = select(assertion.query, &graph, scope_id, &bindings)?.len();
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
