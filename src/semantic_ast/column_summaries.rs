//! Column View summary projection over parsed semantic sections.

use super::{
    ColumnSummaryCell, ColumnSummaryOperatorKind, ColumnSummaryPlan, ColumnSummaryResult,
    ColumnSummaryRow, ColumnSummaryStatus, ColumnSummaryValueSource, ColumnSummaryWarning,
    ColumnSummaryWarningKind, ColumnViewColumn, ColumnViewRecord, ColumnViewScope, Document,
    OrgDuration, ParsedAnnotation, Property, Section, SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Projects non-mutating Column View summary plans from parsed `COLUMNS` metadata.
    ///
    /// Org can write computed summaries back into parent property drawers. This
    /// API intentionally does not mutate source. It exposes the parsed operator,
    /// collected row values, and supported computed results so downstream tools
    /// can render or lint Column View behavior explicitly.
    pub fn column_summary_plans(&self) -> Vec<ColumnSummaryPlan> {
        self.column_view_records()
            .into_iter()
            .map(|record| self.column_summary_plan(record))
            .collect()
    }

    fn column_summary_plan(&self, declaration: ColumnViewRecord) -> ColumnSummaryPlan {
        let mut warnings = Vec::new();
        let rows = match &declaration.scope {
            ColumnViewScope::DocumentKeyword | ColumnViewScope::DocumentProperty => self
                .sections
                .iter()
                .map(|section| column_summary_row(section, &declaration.columns))
                .collect(),
            ColumnViewScope::SectionProperty { .. } => {
                match find_section_for_column_view(&self.sections, &declaration) {
                    Some(section) => {
                        let mut row = column_summary_row(section, &declaration.columns);
                        if let ColumnViewScope::SectionProperty { outline_path, .. } =
                            &declaration.scope
                        {
                            apply_declared_outline_path(&mut row, outline_path);
                        }
                        vec![row]
                    }
                    None => {
                        warnings.push(ColumnSummaryWarning {
                            kind: ColumnSummaryWarningKind::MissingSectionScope,
                            message: "section-scoped COLUMNS declaration could not be matched back to its section".to_string(),
                        });
                        Vec::new()
                    }
                }
            }
        };
        let summaries = match (&declaration.scope, rows.as_slice()) {
            (ColumnViewScope::SectionProperty { .. }, [row]) => row.summaries.clone(),
            _ => column_summary_results(&declaration.columns, &rows),
        };
        ColumnSummaryPlan {
            declaration,
            rows,
            summaries,
            warnings,
        }
    }
}

fn find_section_for_column_view<'a>(
    sections: &'a [Section<ParsedAnnotation>],
    declaration: &ColumnViewRecord,
) -> Option<&'a Section<ParsedAnnotation>> {
    for section in sections {
        if section.properties.iter().any(|property| {
            property.key.eq_ignore_ascii_case("COLUMNS")
                && u32::from(property.ann.range.start()) == declaration.source.range_start
                && u32::from(property.ann.range.end()) == declaration.source.range_end
        }) {
            return Some(section);
        }
        if let Some(found) = find_section_for_column_view(&section.subsections, declaration) {
            return Some(found);
        }
    }
    None
}

fn column_summary_row(
    section: &Section<ParsedAnnotation>,
    columns: &[ColumnViewColumn],
) -> ColumnSummaryRow {
    let children = section
        .subsections
        .iter()
        .map(|subsection| column_summary_row(subsection, columns))
        .collect::<Vec<_>>();
    let cells = columns
        .iter()
        .map(|column| column_summary_cell(section, column))
        .collect::<Vec<_>>();
    let summaries = column_summary_results(columns, &children);
    ColumnSummaryRow {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path: Vec::new(),
        level: section.level,
        title: section.raw_title.trim().to_string(),
        cells,
        summaries,
        children,
    }
    .with_outline_path()
}

impl ColumnSummaryRow {
    fn with_outline_path(mut self) -> Self {
        apply_outline_path(&mut self, &mut Vec::new());
        self
    }
}

fn apply_outline_path(row: &mut ColumnSummaryRow, parent: &mut Vec<String>) {
    parent.push(row.title.clone());
    row.outline_path = parent.clone();
    for child in &mut row.children {
        apply_outline_path(child, parent);
    }
    parent.pop();
}

fn column_summary_cell(
    section: &Section<ParsedAnnotation>,
    column: &ColumnViewColumn,
) -> ColumnSummaryCell {
    let property = column.property.clone();
    if let Some(value) = special_column_value(section, property.as_str()) {
        return ColumnSummaryCell {
            property,
            value: Some(value),
            source: ColumnSummaryValueSource::SpecialProperty,
        };
    }
    if let Some(property_value) = property_value(&section.properties, property.as_str()) {
        return ColumnSummaryCell {
            property,
            value: Some(property_value.to_string()),
            source: ColumnSummaryValueSource::LocalProperty,
        };
    }
    if let Some(property_value) = property_value(&section.effective_properties, property.as_str()) {
        return ColumnSummaryCell {
            property,
            value: Some(property_value.to_string()),
            source: ColumnSummaryValueSource::InheritedProperty,
        };
    }
    ColumnSummaryCell {
        property,
        value: None,
        source: ColumnSummaryValueSource::Missing,
    }
}

fn special_column_value(section: &Section<ParsedAnnotation>, property: &str) -> Option<String> {
    match property {
        "ITEM" => Some(section.raw_title.trim().to_string()),
        "TODO" => section.todo.as_ref().map(|todo| todo.name.clone()),
        "PRIORITY" => Some(section.priority.effective_text()),
        "TAGS" => Some(section.tags.join(":")),
        "ALLTAGS" => Some(section.effective_tags.join(":")),
        "LEVEL" => Some(section.level.to_string()),
        "CLOSED" => section
            .planning
            .closed
            .as_ref()
            .map(|timestamp| timestamp.raw.clone()),
        "SCHEDULED" => section
            .planning
            .scheduled
            .as_ref()
            .map(|timestamp| timestamp.raw.clone()),
        "DEADLINE" => section
            .planning
            .deadline
            .as_ref()
            .map(|timestamp| timestamp.raw.clone()),
        _ => None,
    }
}

fn property_value<'a>(properties: &'a [Property<ParsedAnnotation>], key: &str) -> Option<&'a str> {
    properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.as_str())
}

fn column_summary_results(
    columns: &[ColumnViewColumn],
    rows: &[ColumnSummaryRow],
) -> Vec<ColumnSummaryResult> {
    columns
        .iter()
        .filter(|column| column.summary_operator.is_some())
        .map(|column| column_summary_result(column, rows))
        .collect()
}

fn column_summary_result(
    column: &ColumnViewColumn,
    rows: &[ColumnSummaryRow],
) -> ColumnSummaryResult {
    let operator = column.summary_operator.clone().unwrap_or_default();
    let kind = column_summary_operator_kind(operator.as_str());
    let inputs = rows
        .iter()
        .filter_map(|row| row_summary_input(row, column))
        .collect::<Vec<_>>();
    let input_count = inputs.len();
    let (value, parsed_input_count, status) = if is_special_summary_property(&column.property) {
        (None, 0, ColumnSummaryStatus::IgnoredSpecialProperty)
    } else if input_count == 0 {
        (None, 0, ColumnSummaryStatus::NoInputs)
    } else if matches!(
        kind,
        ColumnSummaryOperatorKind::AgeMin
            | ColumnSummaryOperatorKind::AgeMax
            | ColumnSummaryOperatorKind::AgeMean
            | ColumnSummaryOperatorKind::Custom
    ) {
        (None, 0, ColumnSummaryStatus::Unsupported)
    } else {
        compute_summary(kind, &inputs, column.summary_format.as_deref())
    };
    ColumnSummaryResult {
        column: column.clone(),
        operator,
        kind,
        format: column.summary_format.clone(),
        value,
        input_count,
        parsed_input_count,
        status,
    }
}

fn row_summary_input(row: &ColumnSummaryRow, column: &ColumnViewColumn) -> Option<String> {
    row.summaries
        .iter()
        .find(|summary| {
            summary.column.property == column.property
                && summary.operator == column.summary_operator.clone().unwrap_or_default()
                && summary.status == ColumnSummaryStatus::Computed
        })
        .and_then(|summary| summary.value.clone())
        .or_else(|| {
            row.cells
                .iter()
                .find(|cell| cell.property == column.property)
                .and_then(|cell| cell.value.clone())
        })
}

fn is_special_summary_property(property: &str) -> bool {
    matches!(
        property,
        "ITEM"
            | "TODO"
            | "PRIORITY"
            | "TAGS"
            | "ALLTAGS"
            | "LEVEL"
            | "CLOSED"
            | "SCHEDULED"
            | "DEADLINE"
    )
}

fn column_summary_operator_kind(operator: &str) -> ColumnSummaryOperatorKind {
    match operator {
        "+" => ColumnSummaryOperatorKind::NumericSum,
        "$" => ColumnSummaryOperatorKind::Currency,
        "min" => ColumnSummaryOperatorKind::NumericMin,
        "max" => ColumnSummaryOperatorKind::NumericMax,
        "mean" => ColumnSummaryOperatorKind::NumericMean,
        "X" => ColumnSummaryOperatorKind::CheckboxState,
        "X/" => ColumnSummaryOperatorKind::CheckboxCount,
        "X%" => ColumnSummaryOperatorKind::CheckboxPercent,
        ":" => ColumnSummaryOperatorKind::DurationSum,
        ":min" => ColumnSummaryOperatorKind::DurationMin,
        ":max" => ColumnSummaryOperatorKind::DurationMax,
        ":mean" => ColumnSummaryOperatorKind::DurationMean,
        "@min" => ColumnSummaryOperatorKind::AgeMin,
        "@max" => ColumnSummaryOperatorKind::AgeMax,
        "@mean" => ColumnSummaryOperatorKind::AgeMean,
        "est+" => ColumnSummaryOperatorKind::Estimate,
        _ => ColumnSummaryOperatorKind::Custom,
    }
}

fn compute_summary(
    kind: ColumnSummaryOperatorKind,
    inputs: &[String],
    format: Option<&str>,
) -> (Option<String>, usize, ColumnSummaryStatus) {
    match kind {
        ColumnSummaryOperatorKind::NumericSum => {
            compute_numeric(inputs, format, |values| values.iter().sum())
        }
        ColumnSummaryOperatorKind::Currency => {
            compute_numeric(inputs, Some("%.2f"), |values| values.iter().sum())
        }
        ColumnSummaryOperatorKind::NumericMin => compute_numeric(inputs, format, |values| {
            values.iter().copied().fold(f64::INFINITY, f64::min)
        }),
        ColumnSummaryOperatorKind::NumericMax => compute_numeric(inputs, format, |values| {
            values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        }),
        ColumnSummaryOperatorKind::NumericMean => compute_numeric(inputs, format, |values| {
            values.iter().sum::<f64>() / values.len() as f64
        }),
        ColumnSummaryOperatorKind::CheckboxState => compute_checkbox(inputs, checkbox_state),
        ColumnSummaryOperatorKind::CheckboxCount => compute_checkbox(inputs, checkbox_count),
        ColumnSummaryOperatorKind::CheckboxPercent => compute_checkbox(inputs, checkbox_percent),
        ColumnSummaryOperatorKind::DurationSum => {
            compute_duration(inputs, |values| values.iter().sum())
        }
        ColumnSummaryOperatorKind::DurationMin => compute_duration(inputs, |values| {
            values.iter().copied().min().unwrap_or_default()
        }),
        ColumnSummaryOperatorKind::DurationMax => compute_duration(inputs, |values| {
            values.iter().copied().max().unwrap_or_default()
        }),
        ColumnSummaryOperatorKind::DurationMean => compute_duration(inputs, |values| {
            let total: u64 = values.iter().sum();
            ((total as f64) / values.len() as f64).round() as u64
        }),
        ColumnSummaryOperatorKind::Estimate => compute_estimate(inputs),
        ColumnSummaryOperatorKind::AgeMin
        | ColumnSummaryOperatorKind::AgeMax
        | ColumnSummaryOperatorKind::AgeMean
        | ColumnSummaryOperatorKind::Custom => (None, 0, ColumnSummaryStatus::Unsupported),
    }
}

fn compute_numeric(
    inputs: &[String],
    format: Option<&str>,
    summarize: impl Fn(&[f64]) -> f64,
) -> (Option<String>, usize, ColumnSummaryStatus) {
    let values = inputs
        .iter()
        .filter_map(|input| input.trim().parse::<f64>().ok())
        .collect::<Vec<_>>();
    if values.is_empty() {
        return (None, 0, ColumnSummaryStatus::UnparsedInputs);
    }
    let value = summarize(&values);
    (
        Some(format_number(value, format)),
        values.len(),
        ColumnSummaryStatus::Computed,
    )
}

fn format_number(value: f64, format: Option<&str>) -> String {
    if let Some(precision) = format.and_then(decimal_precision) {
        return format!("{value:.precision$}");
    }
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        let formatted = format!("{value:.6}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn decimal_precision(format: &str) -> Option<usize> {
    let format = format.trim();
    let after_dot = format.strip_prefix("%.")?;
    let digits = after_dot.strip_suffix('f')?;
    digits.parse().ok()
}

fn compute_checkbox(
    inputs: &[String],
    render: impl Fn(usize, usize) -> String,
) -> (Option<String>, usize, ColumnSummaryStatus) {
    let total = inputs.len();
    if total == 0 {
        return (None, 0, ColumnSummaryStatus::NoInputs);
    }
    let done = inputs
        .iter()
        .filter(|input| checkbox_done(input.as_str()))
        .count();
    (
        Some(render(done, total)),
        total,
        ColumnSummaryStatus::Computed,
    )
}

fn checkbox_done(value: &str) -> bool {
    let value = value.trim();
    value == "[X]" || value == "[100%]" || checkbox_count_done(value)
}

fn checkbox_count_done(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return false;
    };
    let Some((done, total)) = inner.split_once('/') else {
        return false;
    };
    done.trim()
        .parse::<usize>()
        .ok()
        .zip(total.trim().parse::<usize>().ok())
        .is_some_and(|(done, total)| total > 0 && done == total)
}

fn checkbox_state(done: usize, total: usize) -> String {
    if done == total {
        "[X]".to_string()
    } else if done > 0 {
        "[-]".to_string()
    } else {
        "[ ]".to_string()
    }
}

fn checkbox_count(done: usize, total: usize) -> String {
    format!("[{done}/{total}]")
}

fn checkbox_percent(done: usize, total: usize) -> String {
    let percent = done
        .saturating_mul(100)
        .saturating_add(total / 2)
        .checked_div(total)
        .unwrap_or(0);
    format!("[{percent}%]")
}

fn compute_duration(
    inputs: &[String],
    summarize: impl Fn(&[u64]) -> u64,
) -> (Option<String>, usize, ColumnSummaryStatus) {
    let values = inputs
        .iter()
        .filter_map(|input| {
            OrgDuration::parse(input.as_str()).map(|duration| duration.total_seconds)
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        return (None, 0, ColumnSummaryStatus::UnparsedInputs);
    }
    (
        Some(format_duration(summarize(&values))),
        values.len(),
        ColumnSummaryStatus::Computed,
    )
}

fn apply_declared_outline_path(row: &mut ColumnSummaryRow, root_path: &[String]) {
    row.outline_path = root_path.to_vec();
    for child in &mut row.children {
        let mut child_path = root_path.to_vec();
        apply_outline_path(child, &mut child_path);
    }
}

fn format_duration(seconds: u64) -> String {
    let minutes = (seconds + 30) / 60;
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

fn compute_estimate(inputs: &[String]) -> (Option<String>, usize, ColumnSummaryStatus) {
    let values = inputs
        .iter()
        .filter_map(|input| estimate_value(input))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return (None, 0, ColumnSummaryStatus::UnparsedInputs);
    }
    let (mean, variance) = values.iter().fold((0.0, 0.0), |(mean, variance), value| {
        (mean + value.mean, variance + value.variance)
    });
    let deviation = variance.sqrt();
    (
        Some(format!("{:.0}-{:.0}", mean - deviation, mean + deviation)),
        values.len(),
        ColumnSummaryStatus::Computed,
    )
}

#[derive(Clone, Copy)]
struct EstimateValue {
    mean: f64,
    variance: f64,
}

fn estimate_value(input: &str) -> Option<EstimateValue> {
    let input = input.trim();
    let values = input
        .split('-')
        .map(str::trim)
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    match values.as_slice() {
        [value] => Some(EstimateValue {
            mean: *value,
            variance: 0.0,
        }),
        [low, high] => {
            let mean = (low + high) / 2.0;
            Some(EstimateValue {
                mean,
                variance: ((low * low + high * high) / 2.0) - (mean * mean),
            })
        }
        _ => None,
    }
}
