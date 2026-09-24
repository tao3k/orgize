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
    At,
    ChildOf,
    DescendantOf,
}

/// Comparison applied to an Org Element field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractFieldMatch {
    Exact,
    Contains,
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
    pub field_match: ContractFieldMatch,
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
}

fn descendant_intervals(targets: &[usize], subtree_end: &[usize]) -> Vec<(usize, usize)> {
    let mut ranges: Vec<_> = targets
        .iter()
        .filter_map(|&target| {
            let start = target + 1;
            let end = subtree_end[target];
            (start < end).then_some((start, end))
        })
        .collect();
    ranges.sort_unstable_by_key(|&(start, _)| start);
    let mut merged = Vec::<(usize, usize)>::with_capacity(ranges.len());
    for (start, end) in ranges {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        merged.push((start, end));
    }
    merged
}

fn in_intervals(node: usize, intervals: &[(usize, usize)]) -> bool {
    let position = intervals.partition_point(|&(start, _)| start <= node);
    position > 0 && node < intervals[position - 1].1
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

fn field_matches(record: &GraphRecord, query: ContractQueryRule) -> bool {
    let (Some(name), Some(expected)) = (query.field_name, query.field_value) else {
        return query.field_name.is_none();
    };
    record
        .fields
        .iter()
        .filter(|field| field.name == name)
        .any(|field| match query.field_match {
            ContractFieldMatch::Exact => field.value == expected,
            ContractFieldMatch::Contains => field.value.contains(expected),
        })
}

fn select(
    query: ContractQueryRule,
    graph: &GraphIndex<'_>,
    scope_id: usize,
    bindings: &HashMap<&'static str, Vec<usize>>,
) -> Result<Vec<usize>, ContractExecutionError> {
    let targets = target_ids(query, scope_id, bindings)?;
    let child_targets: HashSet<_> = targets.iter().copied().collect();
    let descendant_ranges = if query.relation == ContractRelation::DescendantOf {
        descendant_intervals(&targets, &graph.subtree_end)
    } else {
        Vec::new()
    };
    let mut matches = Vec::new();
    for record in graph.records {
        if !graph.descendant_or_self(scope_id, record.id)
            || record.kind != query.node_kind
            || !field_matches(record, query)
        {
            continue;
        }
        let related = match query.relation {
            ContractRelation::Any => true,
            ContractRelation::At => child_targets.contains(&record.id),
            ContractRelation::ChildOf => record
                .parent_id
                .is_some_and(|parent| child_targets.contains(&parent)),
            ContractRelation::DescendantOf => in_intervals(record.id, &descendant_ranges),
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

#[cfg(test)]
#[path = "../tests/unit/contract_feature.rs"]
mod tests;
