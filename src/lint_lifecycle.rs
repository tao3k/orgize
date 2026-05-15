//! Lifecycle and archive lint checks.

use crate::{
    ast::{AstRef, LifecycleRecordKind, ParsedAnnotation, ParsedAst, Property},
    lint_model::{location_for_range, LintFinding, LintSeverity},
};

pub(crate) fn lifecycle_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    push_malformed_logbook_findings(document, source, &mut findings);
    push_archive_location_findings(document, source, &mut findings);
    findings
}

fn push_malformed_logbook_findings(
    document: &ParsedAst,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for record in document.lifecycle_records() {
        let LifecycleRecordKind::MalformedLogbook { reason } = record.kind else {
            continue;
        };
        findings.push(LintFinding {
            code: "ORG014",
            severity: LintSeverity::Warning,
            message: format!("malformed LOGBOOK lifecycle line: {reason}"),
            location: location_for_range(source, record.ann.range),
        });
    }
}

fn push_archive_location_findings(
    document: &ParsedAst,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for location in &document.archive_locations {
        if location.is_empty() {
            findings.push(LintFinding {
                code: "ORG015",
                severity: LintSeverity::Warning,
                message: "#+ARCHIVE keyword has no archive destination".to_string(),
                location: location_for_range(source, location.ann.range),
            });
        }
    }
    push_archive_property_findings(&document.properties, source, findings);
    document.visit(|node| match node {
        AstRef::Section(section) => {
            push_archive_property_findings(&section.properties, source, findings);
        }
        AstRef::Inlinetask(task) => {
            push_archive_property_findings(&task.properties, source, findings);
        }
        _ => {}
    });
}

fn push_archive_property_findings(
    properties: &[Property<ParsedAnnotation>],
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    for property in properties {
        if property.key.eq_ignore_ascii_case("ARCHIVE") && property.value.trim().is_empty() {
            findings.push(LintFinding {
                code: "ORG015",
                severity: LintSeverity::Warning,
                message: "ARCHIVE property has no archive destination".to_string(),
                location: location_for_range(source, property.ann.range),
            });
        }
    }
}
