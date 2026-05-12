//! Conservative Org source formatter.

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
        lines = align_table_runs(&lines);
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

fn align_table_runs(lines: &[String]) -> Vec<String> {
    let mut output = Vec::with_capacity(lines.len());
    let mut index = 0;
    let mut in_block = false;

    while index < lines.len() {
        let line = &lines[index];
        if is_block_begin(line) {
            in_block = true;
            output.push(line.clone());
            index += 1;
            continue;
        }
        if in_block {
            if is_block_end(line) {
                in_block = false;
            }
            output.push(line.clone());
            index += 1;
            continue;
        }
        if !is_table_line(line) {
            output.push(line.clone());
            index += 1;
            continue;
        }

        let start = index;
        while index < lines.len() && is_table_line(&lines[index]) {
            index += 1;
        }
        output.extend(format_table_run(&lines[start..index]));
    }

    output
}

fn is_block_begin(line: &str) -> bool {
    line.trim_start()
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+begin_"))
}

fn is_block_end(line: &str) -> bool {
    line.trim_start()
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+end_"))
}

fn is_table_line(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

#[derive(Debug)]
struct TableRow {
    indent: String,
    cells: Vec<String>,
    is_rule: bool,
}

fn format_table_run(lines: &[String]) -> Vec<String> {
    let rows = lines
        .iter()
        .map(|line| parse_table_row(line))
        .collect::<Vec<_>>();
    let column_count = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    if column_count == 0 {
        return lines.to_vec();
    }

    let mut widths = vec![1usize; column_count];
    for row in &rows {
        if row.is_rule {
            continue;
        }
        for (index, cell) in row.cells.iter().enumerate() {
            widths[index] = widths[index].max(cell.chars().count());
        }
    }

    rows.iter()
        .map(|row| render_table_row(row, &widths))
        .collect()
}

fn parse_table_row(line: &str) -> TableRow {
    let bar = line.find('|').unwrap_or_default();
    let indent = line[..bar].to_string();
    let body = &line[bar..];
    let inner = body.trim_matches('|').trim();
    let is_rule = !inner.is_empty()
        && inner
            .chars()
            .all(|ch| matches!(ch, '-' | '+' | '|') || ch.is_whitespace());
    let cells = if is_rule {
        inner
            .split(['+', '|'])
            .map(|cell| cell.trim().to_string())
            .collect()
    } else {
        let parts = body.split('|').collect::<Vec<_>>();
        let end = if body.ends_with('|') {
            parts.len().saturating_sub(1)
        } else {
            parts.len()
        };
        parts[1..end]
            .iter()
            .map(|cell| cell.trim().to_string())
            .collect()
    };

    TableRow {
        indent,
        cells,
        is_rule,
    }
}

fn render_table_row(row: &TableRow, widths: &[usize]) -> String {
    let mut output = String::new();
    output.push_str(&row.indent);
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
