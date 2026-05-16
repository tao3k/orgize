//! Org table formula lint checks.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    AstRef, ElementData, ParsedAnnotation, ParsedAst, Table, TableFormula, TableFormulaAssignment,
};

use super::lint_model::{location_for_range, LintFinding, LintSeverity};

pub(crate) fn table_formula_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    document.visit(|node| {
        let AstRef::Element(element) = node else {
            return;
        };
        let ElementData::Table(table) = &element.data else {
            return;
        };
        findings.extend(table_findings(table, source));
    });
    findings
}

fn table_findings(table: &Table<ParsedAnnotation>, source: &str) -> Vec<LintFinding> {
    let shape = TableShape::from_table(table);
    table
        .parsed_formulas
        .iter()
        .flat_map(|formula| formula_findings(formula, shape, source))
        .collect()
}

fn formula_findings(
    formula: &TableFormula<ParsedAnnotation>,
    shape: TableShape,
    source: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    findings.extend(malformed_formula_findings(formula, source));
    findings.extend(duplicate_lhs_findings(formula, source));
    findings.extend(out_of_bounds_reference_findings(formula, shape, source));
    findings
}

fn malformed_formula_findings(
    formula: &TableFormula<ParsedAnnotation>,
    source: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut seen_messages = BTreeSet::new();
    for assignment in &formula.assignments {
        if !assignment.raw.contains('=') {
            push_unique_finding(
                &mut findings,
                &mut seen_messages,
                "ORG024",
                format!(
                    "table formula assignment `{}` is missing `=`",
                    assignment.raw
                ),
                formula,
                source,
            );
            continue;
        }
        if assignment.lhs.trim().is_empty() {
            push_unique_finding(
                &mut findings,
                &mut seen_messages,
                "ORG024",
                "table formula assignment has an empty left-hand side".to_string(),
                formula,
                source,
            );
        } else if !is_supported_lhs(&assignment.lhs) {
            push_unique_finding(
                &mut findings,
                &mut seen_messages,
                "ORG024",
                format!(
                    "table formula left-hand side `{}` is not an Org table target",
                    assignment.lhs
                ),
                formula,
                source,
            );
        }
        if assignment.rhs.trim().is_empty() {
            push_unique_finding(
                &mut findings,
                &mut seen_messages,
                "ORG024",
                format!(
                    "table formula assignment `{}` has an empty right-hand side",
                    assignment.lhs
                ),
                formula,
                source,
            );
        }
        for message in malformed_remote_messages(&assignment.raw) {
            push_unique_finding(
                &mut findings,
                &mut seen_messages,
                "ORG024",
                message,
                formula,
                source,
            );
        }
    }
    findings
}

fn duplicate_lhs_findings(
    formula: &TableFormula<ParsedAnnotation>,
    source: &str,
) -> Vec<LintFinding> {
    let mut by_lhs = BTreeMap::<&str, Vec<&TableFormulaAssignment>>::new();
    for assignment in &formula.assignments {
        let lhs = assignment.lhs.trim();
        if lhs.is_empty() || !is_supported_lhs(lhs) {
            continue;
        }
        by_lhs.entry(lhs).or_default().push(assignment);
    }

    by_lhs
        .into_iter()
        .filter_map(|(lhs, assignments)| {
            (assignments.len() > 1).then(|| LintFinding {
                code: "ORG025",
                severity: LintSeverity::Warning,
                message: format!(
                    "table formula target `{lhs}` is defined {} times in one TBLFM line",
                    assignments.len()
                ),
                location: location_for_range(source, formula.ann.range),
            })
        })
        .collect()
}

fn out_of_bounds_reference_findings(
    formula: &TableFormula<ParsedAnnotation>,
    shape: TableShape,
    source: &str,
) -> Vec<LintFinding> {
    let mut findings = Vec::new();
    let mut seen_messages = BTreeSet::new();
    for assignment in &formula.assignments {
        for message in out_of_bounds_messages(&assignment.raw, shape) {
            push_unique_finding(
                &mut findings,
                &mut seen_messages,
                "ORG026",
                message,
                formula,
                source,
            );
        }
    }
    findings
}

fn push_unique_finding(
    findings: &mut Vec<LintFinding>,
    seen_messages: &mut BTreeSet<String>,
    code: &'static str,
    message: String,
    formula: &TableFormula<ParsedAnnotation>,
    source: &str,
) {
    if seen_messages.insert(format!("{code}:{message}")) {
        findings.push(LintFinding {
            code,
            severity: LintSeverity::Warning,
            message,
            location: location_for_range(source, formula.ann.range),
        });
    }
}

fn is_supported_lhs(lhs: &str) -> bool {
    let lhs = lhs.trim();
    if let Some(rest) = lhs.strip_prefix('@') {
        return !rest.is_empty()
            && rest.chars().all(|ch| {
                matches!(
                    ch,
                    '-' | '+' | 'I' | '<' | '>' | '0'..='9' | '.' | '$' | '@'
                )
            });
    }

    let Some(rest) = lhs.strip_prefix('$') else {
        return false;
    };
    !rest.is_empty()
        && (rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            || rest.chars().all(|ch| matches!(ch, '<' | '>')))
}

fn malformed_remote_messages(value: &str) -> Vec<String> {
    let mut messages = Vec::new();
    let mut offset = 0usize;
    while let Some(relative) = value[offset..].find("remote(") {
        let start = offset + relative;
        let rest = &value[start..];
        if let Some(end) = remote_reference_end(rest) {
            offset = start + end;
        } else {
            messages.push("remote table reference is missing a closing `)`".to_string());
            break;
        }
    }
    messages
}

fn out_of_bounds_messages(value: &str, shape: TableShape) -> Vec<String> {
    let mut messages = Vec::new();
    let mut index = 0usize;
    while index < value.len() {
        let rest = &value[index..];
        if rest.starts_with("remote(") {
            if let Some(end) = remote_reference_end(rest) {
                index += end;
                continue;
            }
        }

        let ch = rest.chars().next().unwrap();
        if matches!(ch, '$' | '@') {
            let end = table_reference_end(rest);
            let raw = &rest[..end];
            push_reference_shape_messages(raw, shape, &mut messages);
            index += end;
        } else {
            index += ch.len_utf8();
        }
    }
    messages
}

fn push_reference_shape_messages(raw: &str, shape: TableShape, messages: &mut Vec<String>) {
    if raw.len() <= 1 || raw.contains('#') {
        return;
    }
    for endpoint in raw.split("..") {
        if let Some(row) = absolute_reference_number(endpoint, '@') {
            if row == 0 || row > shape.rows {
                messages.push(format!(
                    "table formula row reference `{raw}` points outside {} table rows",
                    shape.rows
                ));
            }
        }
        if let Some(column) = absolute_reference_number(endpoint, '$') {
            if column == 0 || column > shape.columns {
                messages.push(format!(
                    "table formula column reference `{raw}` points outside {} table columns",
                    shape.columns
                ));
            }
        }
    }
}

fn absolute_reference_number(endpoint: &str, marker: char) -> Option<usize> {
    let marker_index = endpoint.find(marker)?;
    let rest = &endpoint[marker_index + marker.len_utf8()..];
    let digits = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

fn remote_reference_end(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in value.char_indices() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(index + ch.len_utf8());
            }
        }
    }
    None
}

fn table_reference_end(value: &str) -> usize {
    value
        .char_indices()
        .skip(1)
        .find(|(_, ch)| {
            !(ch.is_ascii_alphanumeric()
                || matches!(ch, '$' | '@' | '#' | '.' | '<' | '>' | '_' | '+' | '-'))
        })
        .map(|(index, _)| index)
        .unwrap_or(value.len())
}

#[derive(Clone, Copy)]
struct TableShape {
    rows: usize,
    columns: usize,
}

impl TableShape {
    fn from_table(table: &Table<ParsedAnnotation>) -> Self {
        Self {
            rows: table.rows.len(),
            columns: table
                .rows
                .iter()
                .map(|row| row.cells.len())
                .max()
                .unwrap_or_default(),
        }
    }
}
