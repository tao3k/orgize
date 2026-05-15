//! Property-oriented lint rules.

use std::collections::BTreeMap;

use crate::{
    ast::{Inlinetask, ParsedAnnotation, ParsedAst, Property, Section},
    lint_model::{location_for_range, LintFinding, LintSeverity},
};

pub(crate) fn property_drawer_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    push_property_findings(&document.properties, source, &mut findings);
    for section in &document.sections {
        push_section_property_findings(section, source, &mut findings);
    }
    document.visit(|node| {
        if let crate::ast::AstRef::Inlinetask(task) = node {
            push_inlinetask_property_findings(task, source, &mut findings);
        }
    });
    findings
}

fn push_section_property_findings(
    section: &Section<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    push_property_findings(&section.properties, source, findings);
    for child in &section.subsections {
        push_section_property_findings(child, source, findings);
    }
}

fn push_inlinetask_property_findings(
    task: &Inlinetask<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    push_property_findings(&task.properties, source, findings);
}

fn push_property_findings(
    properties: &[Property<ParsedAnnotation>],
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
