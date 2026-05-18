//! Org-native SDD lint checks.

use std::collections::{BTreeMap, BTreeSet};

use super::lint_model::{location_for_range, LintFinding, LintSeverity};
use crate::ast::{Element, ElementData, ParsedAnnotation, ParsedAst, Section};

/// Lints Org-native SDD nodes and requirement structure.
pub(crate) fn sdd_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let records = document.sdd_node_records();
    let ids = records
        .iter()
        .filter_map(|record| record.id.as_deref())
        .collect::<BTreeSet<_>>();

    for record in &records {
        match &record.id {
            Some(id) if is_stable_sdd_id(id) => {}
            Some(id) => findings.push(LintFinding {
                code: "ORG031",
                severity: LintSeverity::Error,
                message: format!("SDD node `{}` has malformed ID `{id}`", record.title),
                location: location_for_range(source, record.source_range()),
            }),
            None => findings.push(LintFinding {
                code: "ORG031",
                severity: LintSeverity::Error,
                message: format!("SDD node `{}` is missing an ID property", record.title),
                location: location_for_range(source, record.source_range()),
            }),
        }

        if !record.kind.is_known() {
            findings.push(LintFinding {
                code: "ORG032",
                severity: LintSeverity::Error,
                message: format!(
                    "SDD node `{}` has unsupported SDD_KIND `{}`",
                    record.title,
                    record.kind.as_str()
                ),
                location: location_for_range(source, record.source_range()),
            });
        }

        lint_parent_edge(record, &ids, source, &mut findings);
        lint_kind_metadata(record, source, &mut findings);
    }

    findings.extend(duplicate_sdd_id_findings(&records, source));
    for section in &document.sections {
        collect_sdd_execution_state_findings(section, source, &mut findings);
        collect_requirement_findings(section, false, source, &mut findings);
    }

    findings
}

fn lint_kind_metadata(
    record: &crate::ast::SddNodeRecord,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    let missing = match record.kind {
        crate::ast::SddKind::System => record.concern.is_none().then_some("SDD_CONCERN"),
        crate::ast::SddKind::Capability => record.capability.is_none().then_some("SDD_CAPABILITY"),
        crate::ast::SddKind::View => record.viewpoint.is_none().then_some("SDD_VIEWPOINT"),
        crate::ast::SddKind::Decision => record.rationale.is_none().then_some("SDD_RATIONALE"),
        crate::ast::SddKind::Audit => record.concern.is_none().then_some("SDD_CONCERN"),
        crate::ast::SddKind::Unknown(_) => None,
    };

    if let Some(property) = missing {
        findings.push(LintFinding {
            code: "ORG037",
            severity: LintSeverity::Error,
            message: format!(
                "SDD {} node `{}` is missing architecture metadata `{property}`",
                record.kind.as_str(),
                record.title
            ),
            location: location_for_range(source, record.source_range()),
        });
    }
}

trait SddRange {
    fn source_range(&self) -> rowan::TextRange;
}

impl SddRange for crate::ast::SddNodeRecord {
    fn source_range(&self) -> rowan::TextRange {
        rowan::TextRange::new(self.source.range_start.into(), self.source.range_end.into())
    }
}

fn lint_parent_edge(
    record: &crate::ast::SddNodeRecord,
    ids: &BTreeSet<&str>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    match &record.parent {
        Some(parent) => {
            let Some(target_id) = parent.target_id.as_deref() else {
                findings.push(LintFinding {
                    code: "ORG033",
                    severity: LintSeverity::Error,
                    message: format!(
                        "SDD node `{}` has SDD_PARENT `{}` that is not an Org id link",
                        record.title, parent.raw
                    ),
                    location: location_for_range(source, record.source_range()),
                });
                return;
            };
            if !ids.contains(target_id) {
                findings.push(LintFinding {
                    code: "ORG033",
                    severity: LintSeverity::Error,
                    message: format!(
                        "SDD node `{}` references missing parent ID `{target_id}`",
                        record.title
                    ),
                    location: location_for_range(source, record.source_range()),
                });
            }
        }
        None if !record.kind.can_omit_parent() => findings.push(LintFinding {
            code: "ORG033",
            severity: LintSeverity::Error,
            message: format!("SDD node `{}` is missing SDD_PARENT", record.title),
            location: location_for_range(source, record.source_range()),
        }),
        None => {}
    }
}

fn duplicate_sdd_id_findings(
    records: &[crate::ast::SddNodeRecord],
    source: &str,
) -> Vec<LintFinding> {
    let mut by_id = BTreeMap::<&str, Vec<&crate::ast::SddNodeRecord>>::new();
    for record in records {
        if let Some(id) = record.id.as_deref() {
            by_id.entry(id).or_default().push(record);
        }
    }

    let mut findings = Vec::new();
    for (id, records) in by_id {
        if records.len() < 2 {
            continue;
        }
        let duplicate = records[1];
        findings.push(LintFinding {
            code: "ORG034",
            severity: LintSeverity::Error,
            message: format!("SDD ID `{id}` is used by {} SDD nodes", records.len()),
            location: location_for_range(source, duplicate.source_range()),
        });
    }
    findings
}

fn collect_requirement_findings(
    section: &Section<ParsedAnnotation>,
    inside_sdd: bool,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    let current_inside_sdd = inside_sdd || is_sdd_section(section);
    if current_inside_sdd
        && is_requirement_title(&section.raw_title)
        && !has_direct_scenario_child(section)
    {
        findings.push(LintFinding {
            code: "ORG035",
            severity: LintSeverity::Error,
            message: format!(
                "SDD requirement `{}` has no direct Scenario child heading",
                section.raw_title.trim()
            ),
            location: location_for_range(source, section.ann.range),
        });
    }

    for child in &section.subsections {
        collect_requirement_findings(child, current_inside_sdd, source, findings);
    }
}

fn collect_sdd_execution_state_findings(
    section: &Section<ParsedAnnotation>,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    if is_sdd_section(section) {
        if section.todo.is_some() {
            findings.push(sdd_execution_state_finding(
                section,
                "SDD headings must not carry TODO state; move execution state to an Org task or ExecPlan heading",
                source,
            ));
        }
        if title_has_statistics_cookie(&section.raw_title) {
            findings.push(sdd_execution_state_finding(
                section,
                "SDD headings must not carry progress cookies; move implementation progress to an Org task or ExecPlan heading",
                source,
            ));
        }
        if has_direct_checklist(section) {
            findings.push(sdd_execution_state_finding(
                section,
                "SDD headings must not own direct task checklists; move step tracking to an Org task or ExecPlan heading",
                source,
            ));
        }
        if section.subsections.iter().any(|child| child.todo.is_some()) {
            findings.push(sdd_execution_state_finding(
                section,
                "SDD headings must not own direct TODO child tasks; use architecture child headings or link to an implementation plan",
                source,
            ));
        }
    }

    for child in &section.subsections {
        collect_sdd_execution_state_findings(child, source, findings);
    }
}

fn sdd_execution_state_finding(
    section: &Section<ParsedAnnotation>,
    message: &str,
    source: &str,
) -> LintFinding {
    LintFinding {
        code: "ORG036",
        severity: LintSeverity::Warning,
        message: message.to_string(),
        location: location_for_range(source, section.ann.range),
    }
}

fn title_has_statistics_cookie(title: &str) -> bool {
    title
        .split_whitespace()
        .any(|part| fraction_cookie(part) || percent_cookie(part))
}

fn fraction_cookie(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let Some((done, total)) = inner.split_once('/') else {
        return false;
    };
    !done.is_empty()
        && !total.is_empty()
        && done.chars().all(|ch| ch.is_ascii_digit())
        && total.chars().all(|ch| ch.is_ascii_digit())
}

fn percent_cookie(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix("%]"))
    else {
        return false;
    };
    !inner.is_empty() && inner.chars().all(|ch| ch.is_ascii_digit())
}

fn has_direct_checklist(section: &Section<ParsedAnnotation>) -> bool {
    section
        .children
        .iter()
        .any(|element| has_direct_checklist_element(element))
}

fn has_direct_checklist_element(element: &Element<ParsedAnnotation>) -> bool {
    if let ElementData::List(list) = &element.data {
        return list.items.iter().any(|item| item.checkbox.is_some());
    }
    false
}

fn is_sdd_section(section: &Section<ParsedAnnotation>) -> bool {
    section.tags.iter().any(|tag| tag == "sdd") || local_property(section, "SDD_KIND").is_some()
}

fn local_property<'a>(section: &'a Section<ParsedAnnotation>, key: &str) -> Option<&'a str> {
    section
        .properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.trim())
}

fn is_requirement_title(title: &str) -> bool {
    title.trim_start().starts_with("Requirement:")
}

fn has_direct_scenario_child(section: &Section<ParsedAnnotation>) -> bool {
    section
        .subsections
        .iter()
        .any(|child| child.raw_title.trim_start().starts_with("Scenario:"))
}

fn is_stable_sdd_id(value: &str) -> bool {
    is_uuid(value.trim()) || is_ulid(value.trim())
}

fn is_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            _ if !byte.is_ascii_hexdigit() => return false,
            _ => {}
        }
    }
    true
}

fn is_ulid(value: &str) -> bool {
    const ULID_ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    value.len() == 26
        && value
            .chars()
            .all(|ch| ULID_ALPHABET.contains(ch.to_ascii_uppercase()))
}
