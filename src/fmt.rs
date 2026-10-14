//! Conservative Org source formatter.

use crate::org_aot::parse_org_aot;

/// Formatter options for [`format_org`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatOptions {
    /// Remove spaces and tabs from line ends.
    pub trim_trailing_whitespace: bool,
    /// Align contiguous Org table rows.
    pub align_tables: bool,
    /// Ensure formatted non-empty documents end with one newline.
    pub final_newline: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            trim_trailing_whitespace: true,
            align_tables: true,
            final_newline: true,
        }
    }
}

/// Result of formatting one Org source string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatResult {
    pub output: String,
    pub changed: bool,
}

/// Formats Org source with conservative, source-preserving rules.
///
/// The first formatter lane intentionally avoids broad semantic rewrites. It
/// normalizes trailing horizontal whitespace, Org table alignment, final blank
/// lines, and final EOF newline shape.
pub fn format_org(source: &str, options: &FormatOptions) -> FormatResult {
    let mut lines = source
        .split('\n')
        .map(|line| {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if options.trim_trailing_whitespace {
                line.trim_end_matches([' ', '\t']).to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();

    if source.ends_with('\n') {
        lines.pop();
    }

    if options.align_tables {
        align_tables(&mut lines);
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    let mut output = lines.join("\n");
    if options.final_newline && !output.is_empty() {
        output.push('\n');
    }

    FormatResult {
        changed: output != source,
        output,
    }
}

#[derive(Debug)]
struct TableRow {
    line_index: usize,
    cells: Vec<String>,
    is_rule: bool,
}

fn align_tables(lines: &mut [String]) {
    // This is only a cheap candidate check; the AOT graph decides which lines
    // are actually Org table rows, including inside blocks and table formulas.
    if !lines.iter().any(|line| line.trim_start().starts_with('|')) {
        return;
    }
    let source = lines.join("\n");
    let Ok(document) = parse_org_aot(&source) else {
        return;
    };
    let line_starts = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(offset, _)| offset + 1))
        .collect::<Vec<_>>();
    let records = document.records();

    for table in records.iter().filter(|record| record.kind == "table") {
        let rows = table
            .child_ids
            .iter()
            .filter_map(|&id| {
                let record = &records[id];
                if !matches!(record.kind, "table-row" | "table-rule-row") {
                    return None;
                }
                let start = usize::from(record.range.start());
                let line_index = line_starts.partition_point(|&offset| offset <= start) - 1;
                let line = lines.get(line_index)?;
                let end = usize::from(record.range.end());
                if end > line_starts[line_index] + line.len() + 1 {
                    return None;
                }
                let cells = record
                    .child_ids
                    .iter()
                    .filter_map(|&cell_id| {
                        let cell = &records[cell_id];
                        (cell.kind == "table-cell")
                            .then(|| cell.field("text").unwrap_or_default().trim().to_owned())
                    })
                    .collect();
                Some(TableRow {
                    line_index,
                    cells,
                    is_rule: record.kind == "table-rule-row",
                })
            })
            .collect::<Vec<_>>();
        let column_count = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
        if column_count == 0 {
            continue;
        }
        let mut widths = vec![1usize; column_count];
        for row in rows.iter().filter(|row| !row.is_rule) {
            for (index, cell) in row.cells.iter().enumerate() {
                widths[index] = widths[index].max(cell.chars().count());
            }
        }
        for row in &rows {
            let line = &lines[row.line_index];
            let indent = line.split_once('|').map_or("", |(prefix, _)| prefix);
            lines[row.line_index] = render_table_row(row, indent, &widths);
        }
    }
}

fn render_table_row(row: &TableRow, indent: &str, widths: &[usize]) -> String {
    let mut output = String::new();
    output.push_str(indent);
    output.push('|');

    if row.is_rule {
        for (index, width) in widths.iter().enumerate() {
            output.push_str(&"-".repeat(width + 2));
            output.push(if index + 1 == widths.len() { '|' } else { '+' });
        }
        return output;
    }

    for (index, width) in widths.iter().enumerate() {
        let cell = row.cells.get(index).map(String::as_str).unwrap_or_default();
        output.push(' ');
        output.push_str(cell);
        output.push_str(&" ".repeat(width.saturating_sub(cell.chars().count()) + 1));
        output.push('|');
    }
    output
}
