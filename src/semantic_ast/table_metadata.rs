//! Semantic metadata helpers for Org table projection.

use crate::syntax::{SyntaxKind, SyntaxNode};

use super::TableColumnAlignment;

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
