//! Semantic metadata helpers for Org table projection.

use crate::syntax::{SyntaxKind, SyntaxNode};

use super::{
    Keyword, ParsedAnnotation, TableColumnAlignment, TableFormula, TableFormulaAssignment,
    TableFormulaReference, TableFormulaReferenceKind,
};

pub(super) fn table_column_alignments(rows: &[SyntaxNode]) -> Vec<Option<TableColumnAlignment>> {
    rows.iter()
        .find_map(table_column_alignment_row)
        .unwrap_or_default()
}

fn table_column_alignment_row(row: &SyntaxNode) -> Option<Vec<Option<TableColumnAlignment>>> {
    if row.kind() != SyntaxKind::ORG_TABLE_STANDARD_ROW {
        return None;
    }

    let cells = row
        .children()
        .filter(|cell| cell.kind() == SyntaxKind::ORG_TABLE_CELL)
        .map(|cell| table_column_cookie_alignment(&cell.to_string()))
        .collect::<Option<Vec<_>>>()?;

    cells.iter().any(Option::is_some).then_some(cells)
}

fn table_column_cookie_alignment(cell: &str) -> Option<Option<TableColumnAlignment>> {
    let trimmed = cell.trim();
    let inner = trimmed.strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.is_empty() {
        return None;
    }

    let alignment = match inner.chars().next()? {
        'l' if inner[1..].chars().all(|ch| ch.is_ascii_digit()) => Some(TableColumnAlignment::Left),
        'c' if inner[1..].chars().all(|ch| ch.is_ascii_digit()) => {
            Some(TableColumnAlignment::Center)
        }
        'r' if inner[1..].chars().all(|ch| ch.is_ascii_digit()) => {
            Some(TableColumnAlignment::Right)
        }
        _ if inner.chars().all(|ch| ch.is_ascii_digit()) => None,
        _ => return None,
    };

    Some(alignment)
}

pub(super) fn parsed_table_formulas(
    formulas: &[Keyword<ParsedAnnotation>],
) -> Vec<TableFormula<ParsedAnnotation>> {
    formulas
        .iter()
        .map(|formula| TableFormula {
            ann: formula.ann.clone(),
            raw: formula.value.trim().to_string(),
            assignments: table_formula_assignments(&formula.value),
        })
        .collect()
}

fn table_formula_assignments(value: &str) -> Vec<TableFormulaAssignment> {
    value
        .trim()
        .split("::")
        .filter_map(|raw| {
            let raw = raw.trim();
            (!raw.is_empty()).then(|| table_formula_assignment(raw))
        })
        .collect()
}

fn table_formula_assignment(raw: &str) -> TableFormulaAssignment {
    let (left, right) = raw.split_once('=').unwrap_or((raw, ""));
    let mut rhs_and_flags = right.split(';');
    let rhs = rhs_and_flags.next().unwrap_or("").trim().to_string();
    let flags = rhs_and_flags
        .map(str::trim)
        .filter(|flag| !flag.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let references = table_formula_references(left)
        .into_iter()
        .chain(table_formula_references(&rhs))
        .collect();
    TableFormulaAssignment {
        raw: raw.to_string(),
        lhs: left.trim().to_string(),
        rhs,
        flags,
        references,
    }
}

fn table_formula_references(value: &str) -> Vec<TableFormulaReference> {
    let mut references = Vec::new();
    let mut index = 0;

    while index < value.len() {
        let rest = &value[index..];
        if rest.starts_with("remote(")
            && let Some(end) = remote_reference_end(rest)
        {
            references.push(TableFormulaReference {
                raw: rest[..end].to_string(),
                kind: TableFormulaReferenceKind::Remote,
            });
            index += end;
            continue;
        }

        let ch = rest.chars().next().unwrap();
        if matches!(ch, '$' | '@') {
            let end = table_formula_reference_end(rest);
            let raw = rest[..end].to_string();
            references.push(TableFormulaReference {
                kind: if ch == '$' {
                    TableFormulaReferenceKind::Field
                } else {
                    TableFormulaReferenceKind::Row
                },
                raw,
            });
            index += end;
        } else {
            index += ch.len_utf8();
        }
    }

    references
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

fn table_formula_reference_end(value: &str) -> usize {
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
