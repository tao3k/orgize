//! Table metadata projected from Scheme-classified graph cells.

use super::GraphProjector;
use crate::ast::model::{
    ParsedAnnotation, Table, TableCell, TableColumnAlignment, TableFormula, TableFormulaAssignment,
    TableFormulaReference, TableFormulaReferenceKind, TableRow,
};

impl GraphProjector<'_> {
    pub(super) fn table(&mut self, id: usize) -> Table<ParsedAnnotation> {
        let row_ids = self.record(id).child_ids.clone();
        let column_alignments = self.table_column_alignments(&row_ids);
        let formula_ids = row_ids
            .iter()
            .copied()
            .filter(|&child_id| {
                let child = self.record(child_id);
                child.kind == "keyword"
                    && child
                        .field("key")
                        .is_some_and(|key| key.eq_ignore_ascii_case("TBLFM"))
            })
            .collect::<Vec<_>>();
        let formulas = formula_ids
            .iter()
            .filter_map(|&formula_id| self.keyword(formula_id))
            .collect::<Vec<_>>();
        let parsed_formulas = formula_ids
            .into_iter()
            .filter_map(|formula_id| self.table_formula(formula_id))
            .collect();
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
            formulas,
            parsed_formulas,
        }
    }

    fn table_formula(&self, keyword_id: usize) -> Option<TableFormula<ParsedAnnotation>> {
        let keyword = self.record(keyword_id);
        let raw = keyword.field("raw-value")?.trim().to_owned();
        let assignments = keyword
            .child_ids
            .iter()
            .map(|&id| self.record(id))
            .filter(|record| record.kind == "table-formula-value")
            .flat_map(|value| value.child_ids.iter())
            .map(|&id| self.record(id))
            .filter(|record| record.kind == "table-formula-assignment")
            .map(|record| TableFormulaAssignment {
                raw: self.raw(record.range).trim().to_owned(),
                lhs: record
                    .child_ids
                    .iter()
                    .map(|&id| self.record(id))
                    .find(|side| side.kind == "table-formula-lhs")
                    .map(|side| self.raw(side.range).trim().to_owned())
                    .unwrap_or_default(),
                rhs: record
                    .child_ids
                    .iter()
                    .map(|&id| self.record(id))
                    .find(|side| side.kind == "table-formula-rhs")
                    .map(|side| self.raw(side.range).trim().to_owned())
                    .unwrap_or_default(),
                flags: record
                    .fields
                    .iter()
                    .filter(|field| field.name == "flag")
                    .map(|field| field.value.trim().to_owned())
                    .filter(|flag| !flag.is_empty())
                    .collect(),
                references: record
                    .child_ids
                    .iter()
                    .flat_map(|&side_id| self.record(side_id).child_ids.iter())
                    .filter_map(|&reference_id| {
                        let reference = self.record(reference_id);
                        if reference.kind != "table-formula-reference" {
                            return None;
                        }
                        let (raw, kind) = if let Some(raw) = reference.field("field") {
                            (raw, TableFormulaReferenceKind::Field)
                        } else if let Some(raw) = reference.field("row") {
                            (raw, TableFormulaReferenceKind::Row)
                        } else {
                            (
                                reference.field("remote")?,
                                TableFormulaReferenceKind::Remote,
                            )
                        };
                        Some(TableFormulaReference {
                            raw: raw.to_owned(),
                            kind,
                        })
                    })
                    .collect(),
            })
            .collect();
        Some(TableFormula {
            ann: self.annotation(keyword.range),
            raw,
            assignments,
        })
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
