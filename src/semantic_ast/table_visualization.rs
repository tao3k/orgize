//! Non-executing Org Plot and radio-table projection.

use std::collections::BTreeMap;

use super::{
    Document, Element, ElementData, Keyword, Object, ObjectData, ParsedAnnotation, RadioTable,
    RadioTableReceiver, Section, SectionIndexSource, Table, TablePlot, TablePlotType, TableRow,
    TableVisualizationKind, TableVisualizationOption, TableVisualizationOptionKind,
    TableVisualizationPlan, TableVisualizationWarning, TableVisualizationWarningKind,
};

impl Document<ParsedAnnotation> {
    /// Projects `#+PLOT:` and `#+ORGTBL:` table intent without drawing,
    /// translating, or mutating table targets.
    pub fn table_visualization_plans(&self) -> Vec<TableVisualizationPlan<ParsedAnnotation>> {
        let receivers = radio_receivers(self.ann.raw.as_str());
        let mut collector = TableVisualizationCollector {
            receivers,
            plans: Vec::new(),
            pending_radio: None,
            pending_radio_warnings: Vec::new(),
            table_index: 0,
        };
        collector.collect_elements(&self.children, None);
        for section in &self.sections {
            collector.collect_section(section);
        }
        collector.plans
    }
}

struct TableVisualizationCollector {
    receivers: BTreeMap<String, RadioTableReceiver>,
    plans: Vec<TableVisualizationPlan<ParsedAnnotation>>,
    pending_radio: Option<RadioTable<ParsedAnnotation>>,
    pending_radio_warnings: Vec<TableVisualizationWarning>,
    table_index: usize,
}

impl TableVisualizationCollector {
    fn collect_section(&mut self, section: &Section<ParsedAnnotation>) {
        let source = Some(SectionIndexSource::from_annotation(&section.ann));
        self.collect_elements(&section.children, source);
        for subsection in &section.subsections {
            self.collect_section(subsection);
        }
    }

    fn collect_elements(
        &mut self,
        elements: &[Element<ParsedAnnotation>],
        source: Option<SectionIndexSource>,
    ) {
        for element in elements {
            if let ElementData::Keyword(keyword) = &element.data
                && keyword.key.eq_ignore_ascii_case("ORGTBL")
            {
                let (radio, warnings) = radio_table(keyword, &self.receivers);
                self.pending_radio = radio;
                self.pending_radio_warnings = warnings;
                continue;
            }

            match &element.data {
                ElementData::Table(table) => {
                    self.collect_org_table(element, table, source.clone());
                }
                ElementData::TableEl { raw } => {
                    self.collect_table_el(element, raw, source.clone());
                }
                ElementData::Drawer(drawer) => {
                    self.collect_elements(&drawer.children, source.clone())
                }
                ElementData::List(list) => {
                    for item in &list.items {
                        self.collect_elements(&item.children, source.clone());
                    }
                }
                ElementData::Block(block) => self.collect_elements(&block.children, source.clone()),
                ElementData::FootnoteDef(footnote) => {
                    self.collect_elements(&footnote.children, source.clone());
                }
                ElementData::Inlinetask(task) => {
                    self.collect_elements(&task.children, source.clone());
                }
                ElementData::Paragraph(_)
                | ElementData::Keyword(_)
                | ElementData::BabelCall(_)
                | ElementData::Clock(_)
                | ElementData::PropertyDrawer(_)
                | ElementData::Comment(_)
                | ElementData::FixedWidth(_)
                | ElementData::Rule
                | ElementData::LatexEnvironment(_)
                | ElementData::Unknown { .. } => {}
            }
        }
    }

    fn collect_org_table(
        &mut self,
        element: &Element<ParsedAnnotation>,
        table: &Table<ParsedAnnotation>,
        source: Option<SectionIndexSource>,
    ) {
        self.table_index += 1;
        let (plot, mut warnings) = plot_keyword(&element.affiliated_keywords);
        let radio = self.pending_radio.take();
        warnings.append(&mut self.pending_radio_warnings);
        if plot.is_none() && radio.is_none() {
            return;
        }
        let shape = table_shape(table);
        self.plans.push(TableVisualizationPlan {
            ann: element.ann.clone(),
            source,
            table_index: self.table_index,
            kind: TableVisualizationKind::OrgTable,
            row_count: shape.row_count,
            column_count: shape.column_count,
            header: shape.header,
            column_alignments: table.column_alignments.clone(),
            plot,
            radio,
            warnings,
        });
    }

    fn collect_table_el(
        &mut self,
        element: &Element<ParsedAnnotation>,
        raw: &str,
        source: Option<SectionIndexSource>,
    ) {
        self.table_index += 1;
        let (plot, mut warnings) = plot_keyword(&element.affiliated_keywords);
        let radio = self.pending_radio.take();
        warnings.append(&mut self.pending_radio_warnings);
        if plot.is_none() && radio.is_none() {
            return;
        }
        self.plans.push(TableVisualizationPlan {
            ann: element.ann.clone(),
            source,
            table_index: self.table_index,
            kind: TableVisualizationKind::TableEl,
            row_count: raw.lines().filter(|line| !line.trim().is_empty()).count(),
            column_count: 0,
            header: Vec::new(),
            column_alignments: Vec::new(),
            plot,
            radio,
            warnings,
        });
    }
}

struct TableShape {
    row_count: usize,
    column_count: usize,
    header: Vec<String>,
}

fn table_shape(table: &Table<ParsedAnnotation>) -> TableShape {
    let row_count = table.rows.len();
    let column_count = table
        .rows
        .iter()
        .map(|row| row.cells.len())
        .max()
        .unwrap_or(0);
    let header = table
        .rows
        .iter()
        .filter(|row| !row.is_rule)
        .filter(|row| !is_alignment_cookie_row(row))
        .map(row_text)
        .find(|cells| cells.iter().any(|cell| !cell.is_empty()))
        .unwrap_or_default();
    TableShape {
        row_count,
        column_count,
        header,
    }
}

fn row_text(row: &TableRow<ParsedAnnotation>) -> Vec<String> {
    row.cells
        .iter()
        .map(|cell| objects_text(&cell.objects).trim().to_string())
        .collect()
}

fn is_alignment_cookie_row(row: &TableRow<ParsedAnnotation>) -> bool {
    !row.cells.is_empty()
        && row.cells.iter().all(|cell| {
            let value = objects_text(&cell.objects);
            let value = value.trim();
            value.starts_with('<') && value.ends_with('>')
        })
}

fn objects_text(objects: &[Object<ParsedAnnotation>]) -> String {
    objects.iter().map(object_text).collect::<Vec<_>>().join("")
}

fn object_text(object: &Object<ParsedAnnotation>) -> String {
    match &object.data {
        ObjectData::Plain(value)
        | ObjectData::Code(value)
        | ObjectData::Verbatim(value)
        | ObjectData::Entity(value)
        | ObjectData::LatexFragment(value)
        | ObjectData::Target(value)
        | ObjectData::RadioTarget(value)
        | ObjectData::StatisticCookie(value) => value.clone(),
        ObjectData::LineBreak => "\n".to_string(),
        ObjectData::Markup { children, .. } => objects_text(children),
        ObjectData::ExportSnippet { value, .. } => value.clone(),
        ObjectData::FootnoteRef { label, .. } => label.clone().unwrap_or_default(),
        ObjectData::Citation(citation) => citation
            .references
            .iter()
            .map(|reference| format!("@{}", reference.id))
            .collect::<Vec<_>>()
            .join(";"),
        ObjectData::Cloze { raw_text, .. } => raw_text.clone(),
        ObjectData::InlineCall { raw, .. }
        | ObjectData::InlineSrc { raw, .. }
        | ObjectData::Unknown { raw, .. } => raw.clone(),
        ObjectData::Link(link) => {
            let description = link.description_or_default();
            if description.is_empty() {
                link.path().to_string()
            } else {
                objects_text(description)
            }
        }
        ObjectData::Macro { name, arguments } => {
            if arguments.is_empty() {
                format!("{{{{{{{name}}}}}}}")
            } else {
                format!("{{{{{{{}({})}}}}}}", name, arguments.join(","))
            }
        }
        ObjectData::Timestamp(timestamp) => format!("{timestamp:?}"),
    }
}

fn plot_keyword(
    keywords: &[Keyword<ParsedAnnotation>],
) -> (
    Option<TablePlot<ParsedAnnotation>>,
    Vec<TableVisualizationWarning>,
) {
    keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case("PLOT"))
        .map(table_plot)
        .unwrap_or((None, Vec::new()))
}

fn table_plot(
    keyword: &Keyword<ParsedAnnotation>,
) -> (
    Option<TablePlot<ParsedAnnotation>>,
    Vec<TableVisualizationWarning>,
) {
    let options = plot_options(keyword.value.as_str());
    let mut warnings = Vec::new();
    let mut title = None;
    let mut plot_type = None;
    let mut with = None;
    let mut file = None;
    let mut index_column = None;
    let mut time_index_column = None;
    let mut dependent_columns = Vec::new();
    let mut transpose = None;

    for option in &options {
        let value = option.value.as_deref();
        match option.kind {
            TableVisualizationOptionKind::Title => title = value.map(ToString::to_string),
            TableVisualizationOptionKind::Type => plot_type = value.map(TablePlotType::new),
            TableVisualizationOptionKind::With => with = value.map(ToString::to_string),
            TableVisualizationOptionKind::File => file = value.map(ToString::to_string),
            TableVisualizationOptionKind::IndexColumn => {
                index_column = parse_positive_usize_option(option, &mut warnings);
            }
            TableVisualizationOptionKind::TimeIndexColumn => {
                time_index_column = parse_positive_usize_option(option, &mut warnings);
            }
            TableVisualizationOptionKind::DependentColumns => {
                dependent_columns = parse_column_list_option(option, &mut warnings);
            }
            TableVisualizationOptionKind::Transpose => {
                transpose = parse_bool_option(option, &mut warnings);
            }
            TableVisualizationOptionKind::Set
            | TableVisualizationOptionKind::Min
            | TableVisualizationOptionKind::Max
            | TableVisualizationOptionKind::Skip
            | TableVisualizationOptionKind::SkipColumns
            | TableVisualizationOptionKind::Splice
            | TableVisualizationOptionKind::Format
            | TableVisualizationOptionKind::Other => {}
        }
    }

    (
        Some(TablePlot {
            ann: keyword.ann.clone(),
            raw: keyword.value.clone(),
            options,
            title,
            plot_type,
            with,
            file,
            index_column,
            time_index_column,
            dependent_columns,
            transpose,
        }),
        warnings,
    )
}

fn radio_table(
    keyword: &Keyword<ParsedAnnotation>,
    receivers: &BTreeMap<String, RadioTableReceiver>,
) -> (
    Option<RadioTable<ParsedAnnotation>>,
    Vec<TableVisualizationWarning>,
) {
    let tokens = split_option_tokens(keyword.value.as_str());
    let mut warnings = Vec::new();
    if tokens
        .first()
        .is_none_or(|token| !token.eq_ignore_ascii_case("SEND"))
        || tokens.len() < 2
    {
        warnings.push(TableVisualizationWarning {
            kind: TableVisualizationWarningKind::InvalidRadioTableDirective,
            message: "ORGTBL keyword must start with `SEND table-name`".to_string(),
        });
        return (None, warnings);
    }
    let name = tokens[1].clone();
    let translator = tokens.get(2).cloned();
    let parameters = radio_options(&tokens[3..]);
    let receiver = receivers.get(name.as_str()).cloned();
    if receiver.is_none() {
        warnings.push(TableVisualizationWarning {
            kind: TableVisualizationWarningKind::MissingRadioReceiver,
            message: format!("radio table `{name}` has no matching RECEIVE marker"),
        });
    }
    (
        Some(RadioTable {
            ann: keyword.ann.clone(),
            raw: keyword.value.clone(),
            name,
            translator,
            parameters,
            receiver,
        }),
        warnings,
    )
}

fn plot_options(value: &str) -> Vec<TableVisualizationOption> {
    split_option_tokens(value)
        .into_iter()
        .filter_map(|token| {
            let (key, value) = token.split_once(':')?;
            let key = key.trim().to_string();
            let value = trim_wrapping_quotes(value);
            Some(TableVisualizationOption {
                kind: plot_option_kind(key.as_str()),
                key,
                value: (!value.is_empty()).then(|| value.to_string()),
                raw: token,
            })
        })
        .collect()
}

fn radio_options(tokens: &[String]) -> Vec<TableVisualizationOption> {
    let mut options = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(key) = token.strip_prefix(':').filter(|key| !key.is_empty()) {
            let mut raw = token.clone();
            let mut value = None;
            if tokens
                .get(index + 1)
                .is_some_and(|next| !next.starts_with(':'))
            {
                let next = &tokens[index + 1];
                raw.push(' ');
                raw.push_str(next);
                value = Some(trim_wrapping_quotes(next).to_string());
                index += 1;
            }
            options.push(TableVisualizationOption {
                kind: radio_option_kind(key),
                key: key.to_string(),
                value,
                raw,
            });
        }
        index += 1;
    }
    options
}

fn plot_option_kind(key: &str) -> TableVisualizationOptionKind {
    match key.to_ascii_lowercase().as_str() {
        "title" => TableVisualizationOptionKind::Title,
        "ind" => TableVisualizationOptionKind::IndexColumn,
        "timeind" => TableVisualizationOptionKind::TimeIndexColumn,
        "dep" | "deps" => TableVisualizationOptionKind::DependentColumns,
        "transpose" | "trans" => TableVisualizationOptionKind::Transpose,
        "type" => TableVisualizationOptionKind::Type,
        "with" => TableVisualizationOptionKind::With,
        "file" => TableVisualizationOptionKind::File,
        "set" => TableVisualizationOptionKind::Set,
        "min" => TableVisualizationOptionKind::Min,
        "max" => TableVisualizationOptionKind::Max,
        _ => TableVisualizationOptionKind::Other,
    }
}

fn radio_option_kind(key: &str) -> TableVisualizationOptionKind {
    match key.to_ascii_lowercase().as_str() {
        "skip" => TableVisualizationOptionKind::Skip,
        "skipcols" => TableVisualizationOptionKind::SkipColumns,
        "splice" => TableVisualizationOptionKind::Splice,
        "fmt" | "efmt" => TableVisualizationOptionKind::Format,
        _ => TableVisualizationOptionKind::Other,
    }
}

fn parse_positive_usize_option(
    option: &TableVisualizationOption,
    warnings: &mut Vec<TableVisualizationWarning>,
) -> Option<usize> {
    let parsed = option
        .value
        .as_deref()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0);
    if parsed.is_none() {
        warnings.push(invalid_plot_warning(option, "expected a positive integer"));
    }
    parsed
}

fn parse_column_list_option(
    option: &TableVisualizationOption,
    warnings: &mut Vec<TableVisualizationWarning>,
) -> Vec<usize> {
    let Some(value) = option.value.as_deref() else {
        warnings.push(invalid_plot_warning(
            option,
            "expected a parenthesized column list",
        ));
        return Vec::new();
    };
    let inner = value
        .trim()
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(value);
    let columns = inner
        .split(|ch: char| ch.is_whitespace() || ch == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<usize>().ok().filter(|value| *value > 0))
        .collect::<Vec<_>>();
    if columns.is_empty() {
        warnings.push(invalid_plot_warning(
            option,
            "expected one or more positive column numbers",
        ));
    }
    columns
}

fn parse_bool_option(
    option: &TableVisualizationOption,
    warnings: &mut Vec<TableVisualizationWarning>,
) -> Option<bool> {
    match option
        .value
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("y" | "yes" | "t" | "true") => Some(true),
        Some("n" | "no" | "nil" | "false") => Some(false),
        _ => {
            warnings.push(invalid_plot_warning(
                option,
                "expected y/yes/t/true or n/no/nil/false",
            ));
            None
        }
    }
}

fn invalid_plot_warning(
    option: &TableVisualizationOption,
    expectation: &str,
) -> TableVisualizationWarning {
    TableVisualizationWarning {
        kind: TableVisualizationWarningKind::InvalidPlotOption,
        message: format!("PLOT option `{}` is invalid: {expectation}", option.raw),
    }
}

fn radio_receivers(source: &str) -> BTreeMap<String, RadioTableReceiver> {
    let mut receivers = BTreeMap::<String, RadioTableReceiver>::new();
    for line in source.lines() {
        if let Some(name) = radio_receiver_marker(line, "BEGIN RECEIVE ORGTBL") {
            receivers
                .entry(name.clone())
                .and_modify(|receiver| receiver.begin_found = true)
                .or_insert(RadioTableReceiver {
                    name,
                    begin_found: true,
                    end_found: false,
                });
        }
        if let Some(name) = radio_receiver_marker(line, "END RECEIVE ORGTBL") {
            receivers
                .entry(name.clone())
                .and_modify(|receiver| receiver.end_found = true)
                .or_insert(RadioTableReceiver {
                    name,
                    begin_found: false,
                    end_found: true,
                });
        }
    }
    receivers
}

fn radio_receiver_marker(line: &str, marker: &str) -> Option<String> {
    let upper = line.to_ascii_uppercase();
    let start = upper.find(marker)? + marker.len();
    let name = line[start..].split_whitespace().next()?;
    let name = name
        .trim_matches(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')));
    (!name.is_empty()).then(|| name.to_string())
}

fn split_option_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut cursor = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;

    while cursor < value.len() {
        let ch = value[cursor..].chars().next().unwrap();
        if start.is_none() && !ch.is_whitespace() {
            start = Some(cursor);
        }
        cursor += ch.len_utf8();
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '"' | '\'') {
            quote = Some(ch);
        } else if quote.is_none() && ch == '(' {
            paren_depth += 1;
        } else if quote.is_none() && ch == ')' {
            paren_depth = paren_depth.saturating_sub(1);
        } else if quote.is_none()
            && paren_depth == 0
            && ch.is_whitespace()
            && let Some(token_start) = start.take()
        {
            let end = value[..cursor].trim_end().len();
            tokens.push(value[token_start..end].to_string());
        }
    }

    if let Some(token_start) = start {
        tokens.push(value[token_start..].trim_end().to_string());
    }
    tokens
}

fn trim_wrapping_quotes(value: &str) -> &str {
    value
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or_else(|| {
            value
                .trim()
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .unwrap_or(value.trim())
        })
}
