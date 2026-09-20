//! Pairing and reference validation for workspace-level Org contracts.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use crate::ast::{OrgContractWorkspaceReference, OrgContractWorkspaceReferenceSource, Section};

use super::org_contract_workspace::{
    MaintainedDocument, PairedDocument, WorkspacePolicy, collect_section_property_values,
    property_values,
};

pub(super) fn validate_pairs(
    root: &Path,
    documents: &[PairedDocument],
    findings: &mut Vec<String>,
) {
    let by_path = documents
        .iter()
        .enumerate()
        .map(|(index, document)| (document.path.as_path(), index))
        .collect::<HashMap<_, _>>();
    let resolved_counterparts = documents
        .iter()
        .map(|document| resolve_counterpart(&document.path, &document.counterpart))
        .collect::<Vec<_>>();
    let mut by_identity: BTreeMap<(&str, &str), Vec<&PairedDocument>> = BTreeMap::new();

    for (document_index, document) in documents.iter().enumerate() {
        by_identity
            .entry((&document.pair.group, &document.semantic_id))
            .or_default()
            .push(document);
        let counterpart = match &resolved_counterparts[document_index] {
            Ok(path) => path,
            Err(error) => {
                findings.push(format!("{}: {error}", document.relative));
                continue;
            }
        };
        let expected_root = root.join(&document.pair.counterpart_root);
        if !counterpart.starts_with(&expected_root) {
            findings.push(format!(
                "{}: counterpart escapes `{}`: {}",
                document.relative, document.pair.counterpart_root, document.counterpart
            ));
            continue;
        }
        let Some(other_index) = by_path.get(counterpart.as_path()) else {
            findings.push(format!(
                "{}: counterpart is not a maintained paired document: {}",
                document.relative, document.counterpart
            ));
            continue;
        };
        let other = &documents[*other_index];
        if other.semantic_id != document.semantic_id {
            findings.push(format!(
                "{}: counterpart has SEMANTIC_ID `{}` instead of `{}`",
                document.relative, other.semantic_id, document.semantic_id
            ));
        }
        if other.pair.group != document.pair.group {
            findings.push(format!(
                "{}: counterpart belongs to pair group `{}` instead of `{}`",
                document.relative, other.pair.group, document.pair.group
            ));
        }
        match &resolved_counterparts[*other_index] {
            Ok(reciprocal) if reciprocal == &document.path => {}
            _ => findings.push(format!(
                "{}: counterpart relation is not reciprocal",
                document.relative
            )),
        }
    }

    for ((group, semantic_id), members) in by_identity {
        if members.len() != 2 {
            findings.push(format!(
                "pair group `{group}` SEMANTIC_ID `{semantic_id}` must identify exactly two documents; found {}",
                members.len()
            ));
            continue;
        }
        for member in &members {
            let own_root = format!("{}/", member.pair.language_root.trim_end_matches('/'));
            let counterpart_root =
                format!("{}/", member.pair.counterpart_root.trim_end_matches('/'));
            if !member.relative.starts_with(&own_root)
                || !members
                    .iter()
                    .any(|candidate| candidate.relative.starts_with(&counterpart_root))
            {
                findings.push(format!(
                    "pair group `{group}` SEMANTIC_ID `{semantic_id}` does not contain one document in each declared language root"
                ));
                break;
            }
        }
        if let Some(property) = members[0].pair.node_identity_property.as_deref() {
            let left = members[0].node_identities.iter().collect::<BTreeSet<_>>();
            let right = members[1].node_identities.iter().collect::<BTreeSet<_>>();
            if left.is_empty()
                || right.is_empty()
                || left.len() != members[0].node_identities.len()
                || right.len() != members[1].node_identities.len()
                || left != right
            {
                findings.push(format!(
                    "pair group `{group}` SEMANTIC_ID `{semantic_id}` must have identical unique paired node identities in {property}"
                ));
            }
        }
        if members[0].pair.node_equality != members[1].pair.node_equality {
            findings.push(format!(
                "pair group `{group}` SEMANTIC_ID `{semantic_id}` must declare identical pair-node equality contracts"
            ));
        } else if let Some(rule) = members[0].pair.node_equality.as_ref() {
            for identity in members[0].node_metadata.keys() {
                for property in &rule.properties {
                    let left = members[0]
                        .node_metadata
                        .get(identity)
                        .and_then(|values| values.get(property));
                    let right = members[1]
                        .node_metadata
                        .get(identity)
                        .and_then(|values| values.get(property));
                    if left != right {
                        findings.push(format!(
                            "pair group `{group}` SEMANTIC_ID `{semantic_id}` paired node `{identity}` must have equal {property} metadata"
                        ));
                    }
                }
            }
        }
        if members[0].pair.document_equality != members[1].pair.document_equality {
            findings.push(format!(
                "pair group `{group}` SEMANTIC_ID `{semantic_id}` must declare identical pair-document equality contracts"
            ));
        } else if let Some(rule) = members[0].pair.document_equality.as_ref() {
            for property in &rule.properties {
                let left = members[0].document_metadata.get(property);
                let right = members[1].document_metadata.get(property);
                if left != right {
                    findings.push(format!(
                        "pair group `{group}` SEMANTIC_ID `{semantic_id}` must have equal document property {property}"
                    ));
                }
            }
        }
    }
}

pub(super) fn validate_references(
    documents: &[MaintainedDocument],
    policy: &WorkspacePolicy,
    findings: &mut Vec<String>,
) {
    let identity_properties = policy
        .routes
        .iter()
        .flat_map(|route| &route.references)
        .map(|rule| rule.identity_property.as_str())
        .collect::<BTreeSet<_>>();
    let identities = identity_properties
        .into_iter()
        .map(|property| {
            let mut values = Vec::new();
            for document in documents {
                collect_section_property_values(&document.document.sections, property, &mut values);
            }
            (property, values.into_iter().collect::<BTreeSet<_>>())
        })
        .collect::<BTreeMap<_, _>>();
    let target_projections = policy
        .routes
        .iter()
        .flat_map(|route| &route.references)
        .flat_map(|rule| {
            rule.reciprocal_property
                .iter()
                .map(|property| (rule.identity_property.clone(), property.clone()))
                .chain(rule.target_property.iter().map(|constraint| {
                    (rule.identity_property.clone(), constraint.property.clone())
                }))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|projection| {
            let mut values = BTreeMap::new();
            for document in documents {
                collect_identity_property_values(
                    &document.document.sections,
                    &projection.0,
                    &projection.1,
                    &mut values,
                );
            }
            (projection, values)
        })
        .collect::<BTreeMap<_, _>>();

    for document in documents {
        let route = &policy.routes[document.route_index];
        for rule in &route.references {
            let identity_values = identities
                .get(rule.identity_property.as_str())
                .expect("identity property was collected");
            match rule.source {
                OrgContractWorkspaceReferenceSource::DocumentProperty => {
                    for value in property_values(&document.document.properties, &rule.property) {
                        validate_reference_tokens(
                            value,
                            rule,
                            identity_values,
                            &document.relative,
                            "document",
                            findings,
                        );
                        validate_target_property_tokens(
                            value,
                            rule,
                            &target_projections,
                            &document.relative,
                            "document",
                            findings,
                        );
                    }
                }
                OrgContractWorkspaceReferenceSource::NodeProperty => {
                    validate_node_references(
                        &document.document.sections,
                        rule,
                        identity_values,
                        &target_projections,
                        &document.relative,
                        findings,
                    );
                }
            }
        }
    }

    let mut acyclic_routes = BTreeMap::<OrgContractWorkspaceReference, BTreeSet<usize>>::new();
    for (route_index, route) in policy.routes.iter().enumerate() {
        for rule in route.references.iter().filter(|rule| rule.acyclic) {
            acyclic_routes
                .entry(rule.clone())
                .or_default()
                .insert(route_index);
        }
    }
    for (rule, route_indexes) in acyclic_routes {
        let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
        for document in documents
            .iter()
            .filter(|document| route_indexes.contains(&document.route_index))
        {
            collect_reference_edges(&document.document.sections, &rule, &mut graph);
        }
        if let Some(cycle) = find_reference_cycle(&graph) {
            findings.push(format!(
                "node property {} must be acyclic; cycle: {}",
                rule.property,
                cycle.join(" -> ")
            ));
        }
    }
}

fn collect_reference_edges<A>(
    sections: &[Section<A>],
    rule: &OrgContractWorkspaceReference,
    graph: &mut BTreeMap<String, BTreeSet<String>>,
) {
    for section in sections {
        let identities = property_values(&section.properties, &rule.identity_property);
        if let [identity] = identities.as_slice() {
            let edges = graph.entry((*identity).to_string()).or_default();
            for value in property_values(&section.properties, &rule.property) {
                edges.extend(
                    value
                        .split_whitespace()
                        .filter(|target| !rule.allowed_values.contains(*target))
                        .map(str::to_string),
                );
            }
        }
        collect_reference_edges(&section.subsections, rule, graph);
    }
}

fn find_reference_cycle(graph: &BTreeMap<String, BTreeSet<String>>) -> Option<Vec<String>> {
    let mut visited = BTreeSet::new();
    let mut active = BTreeSet::new();
    let mut path = Vec::new();
    for node in graph.keys() {
        if let Some(cycle) = visit_reference_node(node, graph, &mut visited, &mut active, &mut path)
        {
            return Some(cycle);
        }
    }
    None
}

fn visit_reference_node(
    node: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
    active: &mut BTreeSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visited.contains(node) {
        return None;
    }
    active.insert(node.to_string());
    path.push(node.to_string());
    if let Some(targets) = graph.get(node) {
        for target in targets {
            if active.contains(target) {
                let start = path.iter().position(|entry| entry == target)?;
                let mut cycle = path[start..].to_vec();
                cycle.push(target.clone());
                return Some(cycle);
            }
            if let Some(cycle) = visit_reference_node(target, graph, visited, active, path) {
                return Some(cycle);
            }
        }
    }
    path.pop();
    active.remove(node);
    visited.insert(node.to_string());
    None
}

type TargetProjections = BTreeMap<(String, String), BTreeMap<String, Vec<BTreeSet<String>>>>;

fn collect_identity_property_values<A>(
    sections: &[Section<A>],
    identity_property: &str,
    value_property: &str,
    values: &mut BTreeMap<String, Vec<BTreeSet<String>>>,
) {
    for section in sections {
        let identities = property_values(&section.properties, identity_property);
        if let [identity] = identities.as_slice()
            && !identity.is_empty()
        {
            let target = property_values(&section.properties, value_property)
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<_>>();
            values
                .entry((*identity).to_string())
                .or_default()
                .push(target);
        }
        collect_identity_property_values(
            &section.subsections,
            identity_property,
            value_property,
            values,
        );
    }
}

fn validate_node_references<A>(
    sections: &[Section<A>],
    rule: &OrgContractWorkspaceReference,
    identity_values: &BTreeSet<String>,
    target_projections: &TargetProjections,
    path: &str,
    findings: &mut Vec<String>,
) {
    for section in sections {
        let source_identities = property_values(&section.properties, &rule.identity_property);
        for value in property_values(&section.properties, &rule.property) {
            if rule.exclude_self
                && let [identity] = source_identities.as_slice()
            {
                for reference in value.split_whitespace() {
                    if reference == *identity {
                        findings.push(format!(
                            "{path}: node `{identity}` property {} must not reference its own identity",
                            rule.property
                        ));
                    }
                }
            }
            validate_reference_tokens(value, rule, identity_values, path, "node", findings);
            validate_target_property_tokens(
                value,
                rule,
                target_projections,
                path,
                "node",
                findings,
            );
            if let Some(reciprocal_property) = rule.reciprocal_property.as_ref()
                && let [identity] = source_identities.as_slice()
            {
                let reciprocal_values = target_projections
                    .get(&(rule.identity_property.clone(), reciprocal_property.clone()))
                    .expect("reciprocal projection was collected");
                for reference in value.split_whitespace() {
                    if rule.allowed_values.contains(reference) {
                        continue;
                    }
                    if !reciprocal_values.get(reference).is_some_and(|instances| {
                        !instances.is_empty()
                            && instances.iter().all(|values| {
                                values.iter().any(|value| {
                                    value.split_whitespace().any(|token| token == *identity)
                                })
                            })
                    }) {
                        findings.push(format!(
                            "{path}: node `{identity}` property {} reference `{reference}` must be reciprocated by target property {reciprocal_property}",
                            rule.property
                        ));
                    }
                }
            }
        }
        validate_node_references(
            &section.subsections,
            rule,
            identity_values,
            target_projections,
            path,
            findings,
        );
    }
}

fn validate_target_property_tokens(
    value: &str,
    rule: &OrgContractWorkspaceReference,
    target_projections: &TargetProjections,
    path: &str,
    scope: &str,
    findings: &mut Vec<String>,
) {
    let Some(constraint) = rule.target_property.as_ref() else {
        return;
    };
    let target_values = target_projections
        .get(&(rule.identity_property.clone(), constraint.property.clone()))
        .expect("target property projection was collected");
    for reference in value.split_whitespace() {
        if rule.allowed_values.contains(reference) {
            continue;
        }
        let satisfies = target_values.get(reference).is_some_and(|instances| {
            !instances.is_empty()
                && instances.iter().all(|values| {
                    !values.is_empty()
                        && values
                            .iter()
                            .all(|value| constraint.allowed_values.contains(value))
                })
        });
        if !satisfies {
            findings.push(format!(
                "{path}: {scope} property {} reference `{reference}` target property {} must be one of {}",
                rule.property,
                constraint.property,
                constraint
                    .allowed_values
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
}

fn validate_reference_tokens(
    value: &str,
    rule: &OrgContractWorkspaceReference,
    identities: &BTreeSet<String>,
    path: &str,
    scope: &str,
    findings: &mut Vec<String>,
) {
    let references = value.split_whitespace().collect::<Vec<_>>();
    let has_allowed_value = references
        .iter()
        .any(|reference| rule.allowed_values.contains(*reference));
    let has_identity_reference = references
        .iter()
        .any(|reference| !rule.allowed_values.contains(*reference));
    if has_allowed_value && has_identity_reference {
        findings.push(format!(
            "{path}: {scope} property {} must not mix allowed sentinel values with identity references",
            rule.property
        ));
    }

    for reference in references {
        if !rule.allowed_values.contains(reference) && !identities.contains(reference) {
            findings.push(format!(
                "{path}: {scope} property {} reference `{reference}` does not resolve to node identity {}",
                rule.property, rule.identity_property
            ));
        }
    }
}

fn resolve_counterpart(owner: &Path, value: &str) -> Result<PathBuf, String> {
    let joined = owner.parent().unwrap_or_else(|| Path::new(".")).join(value);
    lexical_normalize(&joined)
        .ok_or_else(|| format!("counterpart `{value}` escapes the filesystem root"))
}

fn lexical_normalize(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(component.as_os_str()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            std::path::Component::Normal(part) => normalized.push(part),
        }
    }
    Some(normalized)
}
