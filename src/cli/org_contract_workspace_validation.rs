//! Pairing and reference validation for workspace-level Org contracts.

use std::{
    collections::{BTreeMap, BTreeSet},
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
        .map(|document| (document.path.clone(), document))
        .collect::<BTreeMap<_, _>>();
    let mut by_identity: BTreeMap<(&str, &str), Vec<&PairedDocument>> = BTreeMap::new();

    for document in documents {
        by_identity
            .entry((&document.pair.group, &document.semantic_id))
            .or_default()
            .push(document);
        let counterpart = match resolve_counterpart(&document.path, &document.counterpart) {
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
        let Some(other) = by_path.get(&counterpart) else {
            findings.push(format!(
                "{}: counterpart is not a maintained paired document: {}",
                document.relative, document.counterpart
            ));
            continue;
        };
        if other.semantic_id != document.semantic_id {
            findings.push(format!(
                "{}: counterpart has SEMANTIC_ID `{}` instead of `{}`",
                document.relative, other.semantic_id, document.semantic_id
            ));
        }
        match resolve_counterpart(&other.path, &other.counterpart) {
            Ok(reciprocal) if reciprocal == document.path => {}
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
    let reciprocal_projections = policy
        .routes
        .iter()
        .flat_map(|route| &route.references)
        .filter_map(|rule| {
            rule.reciprocal_property
                .as_ref()
                .map(|property| (rule.identity_property.clone(), property.clone()))
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|projection| {
            let mut values = BTreeMap::new();
            for document in documents {
                collect_identity_property_tokens(
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
                    }
                }
                OrgContractWorkspaceReferenceSource::NodeProperty => {
                    validate_node_references(
                        &document.document.sections,
                        rule,
                        identity_values,
                        &reciprocal_projections,
                        &document.relative,
                        findings,
                    );
                }
            }
        }
    }
}

fn collect_identity_property_tokens<A>(
    sections: &[Section<A>],
    identity_property: &str,
    value_property: &str,
    values: &mut BTreeMap<String, BTreeSet<String>>,
) {
    for section in sections {
        let identities = property_values(&section.properties, identity_property);
        if let [identity] = identities.as_slice()
            && !identity.is_empty()
        {
            let target = values.entry((*identity).to_string()).or_default();
            for value in property_values(&section.properties, value_property) {
                target.extend(value.split_whitespace().map(str::to_string));
            }
        }
        collect_identity_property_tokens(
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
    reciprocal_projections: &BTreeMap<(String, String), BTreeMap<String, BTreeSet<String>>>,
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
            if let Some(reciprocal_property) = rule.reciprocal_property.as_ref()
                && let [identity] = source_identities.as_slice()
            {
                let reciprocal_values = reciprocal_projections
                    .get(&(rule.identity_property.clone(), reciprocal_property.clone()))
                    .expect("reciprocal projection was collected");
                for reference in value.split_whitespace() {
                    if rule.allowed_values.contains(reference) {
                        continue;
                    }
                    if !reciprocal_values
                        .get(reference)
                        .is_some_and(|values| values.contains(*identity))
                    {
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
            reciprocal_projections,
            path,
            findings,
        );
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
    for reference in value.split_whitespace() {
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
