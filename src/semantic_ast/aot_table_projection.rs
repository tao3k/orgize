//! Table metadata projected from Scheme-classified graph cells.

use super::GraphProjector;
use crate::ast::model::{ParsedAnnotation, Table, TableCell, TableColumnAlignment, TableRow};

impl GraphProjector<'_> {
    pub(super) fn table(&mut self, id: usize) -> Table<ParsedAnnotation> {
        let row_ids = self.record(id).child_ids.clone();
        let column_alignments = self.table_column_alignments(&row_ids);
        let rows = row_ids
            .into_iter()
            .filter_map(|row_id| {
                let row = self.record(row_id);
                if !matches!(row.kind, "table-row" | "table-rule-row") {
                    return None;
                }
                let row_range = row.range;
                let is_rule = row.kind == "table-rule-row";
                let mut cell_ids = row.child_ids.clone();
                cell_ids.retain(|&cell_id| self.record(cell_id).kind == "table-cell");
                let cells = cell_ids
                    .into_iter()
                    .map(|cell_id| self.table_cell(cell_id))
                    .collect();
                Some(TableRow {
                    ann: self.annotation(row_range),
                    is_rule,
                    cells,
                })
            })
            .collect();
        Table {
            rows,
            column_alignments,
            formulas: Vec::new(),
            parsed_formulas: Vec::new(),
        }
    }

    fn table_cell(&mut self, id: usize) -> TableCell<ParsedAnnotation> {
        let cell = self.record(id);
        let range = cell.range;
        let children = cell.child_ids.clone();
        let text = self.raw(range);
        let trimmed = text.trim();
        let offset = usize::from(range.start()) + text.len() - text.trim_start().len();
        let span = rowan::TextRange::new(
            (offset as u32).into(),
            ((offset + trimmed.len()) as u32).into(),
        );
        let objects = if children.is_empty() || trimmed.is_empty() {
            vec![self.plain(range, trimmed)]
        } else {
            self.objects_in_span(span, &children)
        };
        TableCell {
            ann: self.annotation(range),
            objects,
        }
    }

    fn table_column_alignments(&self, row_ids: &[usize]) -> Vec<Option<TableColumnAlignment>> {
        row_ids
            .iter()
            .find_map(|&row_id| {
                let row = self.record(row_id);
                if row.kind != "table-row" {
                    return None;
                }
                let alignments = row
                    .child_ids
                    .iter()
                    .filter(|&&cell_id| self.record(cell_id).kind == "table-cell")
                    .try_fold(Vec::new(), |mut alignments, &cell_id| {
                        let cookie = crate::org_aot::table_column_cookie_kind(
                            self.record(cell_id).field("text").unwrap_or_default(),
                        );
                        let alignment = match cookie {
                            "left" => Some(TableColumnAlignment::Left),
                            "center" => Some(TableColumnAlignment::Center),
                            "right" => Some(TableColumnAlignment::Right),
                            "width" => None,
                            _ => return None,
                        };
                        alignments.push(alignment);
                        Some(alignments)
                    })?;
                alignments.iter().any(Option::is_some).then_some(alignments)
            })
            .unwrap_or_default()
    }
}
