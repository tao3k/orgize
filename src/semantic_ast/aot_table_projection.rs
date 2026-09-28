//! Table metadata projected from Scheme-classified graph cells.

use super::GraphProjector;
use crate::ast::model::{ParsedAnnotation, Table, TableCell, TableColumnAlignment, TableRow};

impl GraphProjector<'_> {
    pub(super) fn table(&self, id: usize) -> Table<ParsedAnnotation> {
        let row_ids = &self.record(id).child_ids;
        let column_alignments = self.table_column_alignments(row_ids);
        let rows = row_ids
            .iter()
            .filter_map(|&row_id| {
                let row = self.record(row_id);
                if !matches!(row.kind, "table-row" | "table-rule-row") {
                    return None;
                }
                let cells =
                    row.child_ids
                        .iter()
                        .filter_map(|&cell_id| {
                            let cell = self.record(cell_id);
                            (cell.kind == "table-cell").then(|| TableCell {
                                ann: self.annotation(cell.range),
                                objects: vec![self.plain(
                                    cell.range,
                                    cell.field("text").unwrap_or_default().trim(),
                                )],
                            })
                        })
                        .collect();
                Some(TableRow {
                    ann: self.annotation(row.range),
                    is_rule: row.kind == "table-rule-row",
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
