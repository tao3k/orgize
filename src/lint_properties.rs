//! Property-oriented lint rules.

use std::collections::BTreeMap;

use crate::ast::{
    Inlinetask, ParsedAnnotation, ParsedAst, Property, PropertyProfile, PropertySchemaRegistry,
    Section, is_allowed_value_descriptor, property_allowed_values,
};

use super::lint_model::{LintFinding, LintSeverity, location_for_range, location_for_range_bounds};

pub(crate) fn property_drawer_findings(
    document: &ParsedAst,
    source: &str,
    schema_registry: &PropertySchemaRegistry,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let profile = document.property_profile_with_schema_registry(schema_registry);
    push_property_findings(&document.properties, &profile, source, &mut findings);
    push_allowed_value_findings(
        &document.properties,
        &document.properties,
        &profile,
        source,
        &mut findings,
    );
    for section in &document.sections {
        push_section_property_findings(section, &profile, source, &mut findings);
    }
    document.visit(|node| {
        if let crate::ast::AstRef::Inlinetask(task) = node {
            push_inlinetask_property_findings(task, &profile, source, &mut findings);
        }
    });
    push_property_schema_findings(&profile, source, &mut findings);
    findings
}

fn push_section_property_findings(
    section: &Section<ParsedAnnotation>,
    profile: &PropertyProfile,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    push_property_findings(&section.properties, profile, source, findings);
    push_allowed_value_findings(
        &section.properties,
        &section.effective_properties,
        profile,
        source,
        findings,
    );
    for child in &section.subsections {
        push_section_property_findings(child, profile, source, findings);
    }
}

fn push_inlinetask_property_findings(
    task: &Inlinetask<ParsedAnnotation>,
    profile: &PropertyProfile,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    push_property_findings(&task.properties, profile, source, findings);
    push_allowed_value_findings(
        &task.properties,
        &task.properties,
        profile,
        source,
        findings,
    );
}

fn push_property_findings(
    properties: &[Property<ParsedAnnotation>],
    _profile: &PropertyProfile,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    let mut by_key = BTreeMap::<String, Vec<&Property<ParsedAnnotation>>>::new();
    for property in properties {
        push_effort_duration_finding(property, source, findings);
        push_property_typo_finding(property, source, findings);
        by_key
            .entry(property.key.to_ascii_uppercase())
            .or_default()
            .push(property);
    }
    push_duplicate_property_findings(by_key, source, findings);
}

fn push_allowed_value_findings(
    local_properties: &[Property<ParsedAnnotation>],
    effective_properties: &[Property<ParsedAnnotation>],
    profile: &PropertyProfile,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    findings.extend(local_properties.iter().filter_map(|property| {
        allowed_value_finding(property, effective_properties, profile, source)
    }));
}

fn allowed_value_finding(
    property: &Property<ParsedAnnotation>,
    effective_properties: &[Property<ParsedAnnotation>],
    profile: &PropertyProfile,
    source: &str,
) -> Option<LintFinding> {
    if property.value.trim().is_empty() || is_allowed_value_descriptor(&property.key) {
        return None;
    }
    let values = property_allowed_values(effective_properties, profile, &property.key)?;
    if values.iter().any(|value| value == &property.value) {
        return None;
    }
    Some(LintFinding {
        code: "ORG030",
        severity: LintSeverity::Warning,
        message: format!(
            "property `{}` value `{}` is not in allowed values: {}",
            property.key,
            property.value,
            values.join(", ")
        ),
        location: location_for_range(source, property.ann.range),
    })
}

fn push_property_schema_findings(
    profile: &PropertyProfile,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    findings.extend(
        profile
            .schema_applications
            .iter()
            .flat_map(|application| application.findings.iter())
            .map(|finding| LintFinding {
                code: "ORG040",
                severity: LintSeverity::Warning,
                message: finding.message.clone(),
                location: location_for_range_bounds(
                    source,
                    finding.source.range_start as usize,
                    finding.source.range_end as usize,
                ),
            }),
    );
}

fn push_effort_duration_finding(
    property: &Property<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    if property.is_effort() && !property.value.trim().is_empty() && property.duration.is_none() {
        findings.push(LintFinding {
            code: "ORG011",
            severity: LintSeverity::Warning,
            message: format!(
                "EFFORT property value `{}` is not an Org duration",
                property.value
            ),
            location: location_for_range(source, property.ann.range),
        });
    }
}

fn push_property_typo_finding(
    property: &Property<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    let Some(expected) = common_property_typo(&property.key) else {
        return;
    };
    findings.push(LintFinding {
        code: "ORG013",
        severity: LintSeverity::Warning,
        message: format!(
            "property `{}` looks like a typo; use `{expected}` for agenda effort semantics",
            property.key
        ),
        location: location_for_range(source, property.ann.range),
    });
}

fn push_duplicate_property_findings(
    by_key: BTreeMap<String, Vec<&Property<ParsedAnnotation>>>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for (key, definitions) in by_key {
        if definitions.len() < 2 {
            continue;
        }
        let duplicate = definitions[1];
        findings.push(LintFinding {
            code: "ORG012",
            severity: LintSeverity::Warning,
            message: format!(
                "property `{key}` is defined {} times in one local scope",
                definitions.len()
            ),
            location: location_for_range(source, duplicate.ann.range),
        });
    }
}

fn common_property_typo(key: &str) -> Option<&'static str> {
    match key.to_ascii_uppercase().as_str() {
        "EFORT" | "EFFORTS" | "EFFORTT" => Some("EFFORT"),
        _ => None,
    }
}
