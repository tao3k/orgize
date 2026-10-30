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
        let receivers = radio_receivers(self);
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
                | ElementData::DiarySexp(_)
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
            super::org_values::scalar("alignment-cookie", &[&value]) == "true"
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
    let mut warnings = Vec::new();
    let Some(row) = super::org_values::optional("radio-header", &keyword.value) else {
        warnings.push(TableVisualizationWarning {
            kind: TableVisualizationWarningKind::InvalidRadioTableDirective,
            message: "ORGTBL keyword must start with `SEND table-name`".to_owned(),
        });
        return (None, warnings);
    };
    let [name, translator, present]: [String; 3] = row.try_into().expect("native radio header");
    let translator = match present.as_str() {
        "true" => Some(translator),
        "false" => None,
        _ => panic!("native radio translator presence"),
    };
    let parameters = radio_options(&keyword.value);
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
    super::org_values::rows("plot-options", &[value])
        .into_iter()
        .map(|row| {
            let [key, value, raw]: [String; 3] = row.try_into().expect("native plot option arity");
            TableVisualizationOption {
                kind: plot_option_kind(&key),
                key,
                value: (!value.is_empty()).then_some(value),
                raw,
            }
        })
        .collect()
}
fn radio_options(value: &str) -> Vec<TableVisualizationOption> {
    super::org_values::rows("radio-options", &[value])
        .into_iter()
        .map(|row| {
            let [key, value, present, raw]: [String; 4] =
                row.try_into().expect("native radio option arity");
            let value = match present.as_str() {
                "true" => Some(value),
                "false" => None,
                _ => panic!("native radio option presence"),
            };
            TableVisualizationOption {
                kind: radio_option_kind(&key),
                key,
                value,
                raw,
            }
        })
        .collect()
}

fn plot_option_kind(key: &str) -> TableVisualizationOptionKind {
    match super::org_values::scalar("option-kind", &[key, "plot"]).as_str() {
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
    match super::org_values::scalar("option-kind", &[key, "radio"]).as_str() {
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
        .and_then(|value| super::org_values::optional("positive", value))
        .map(|row| row[0].parse::<usize>().expect("native positive integer"))
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
    let columns = super::org_values::rows("column-list", &[value])
        .pop()
        .expect("native columns")
        .into_iter()
        .map(|value| value.parse().expect("native column number"))
        .collect::<Vec<usize>>();
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
    let value = option
        .value
        .as_deref()
        .and_then(|value| super::org_values::optional("plot-bool", value));
    match value.as_deref() {
        Some([value]) if value == "true" => Some(true),
        Some([value]) if value == "false" => Some(false),
        Some(_) => panic!("native plot bool arity/domain"),
        None => {
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

fn radio_receivers(document: &Document<ParsedAnnotation>) -> BTreeMap<String, RadioTableReceiver> {
    // Only Scheme-AOT comment elements can declare receivers; opaque block text
    // and ordinary paragraphs must not create radio-table targets.
    let mut receivers = BTreeMap::<String, RadioTableReceiver>::new();
    collect_radio_receivers_in_elements(&document.children, &mut receivers);
    for section in &document.sections {
        collect_radio_receivers_in_section(section, &mut receivers);
    }
    receivers
}

fn collect_radio_receivers_in_section(
    section: &Section<ParsedAnnotation>,
    receivers: &mut BTreeMap<String, RadioTableReceiver>,
) {
    collect_radio_receivers_in_elements(&section.children, receivers);
    for subsection in &section.subsections {
        collect_radio_receivers_in_section(subsection, receivers);
    }
}

fn collect_radio_receivers_in_elements(
    elements: &[Element<ParsedAnnotation>],
    receivers: &mut BTreeMap<String, RadioTableReceiver>,
) {
    for element in elements {
        match &element.data {
            ElementData::Comment(raw) => collect_radio_receiver_comment(raw, receivers),
            ElementData::Drawer(drawer) => {
                collect_radio_receivers_in_elements(&drawer.children, receivers);
            }
            ElementData::List(list) => {
                for item in &list.items {
                    collect_radio_receivers_in_elements(&item.children, receivers);
                }
            }
            ElementData::Block(block) => {
                collect_radio_receivers_in_elements(&block.children, receivers);
            }
            ElementData::FootnoteDef(footnote) => {
                collect_radio_receivers_in_elements(&footnote.children, receivers);
            }
            ElementData::Inlinetask(task) => {
                collect_radio_receivers_in_elements(&task.children, receivers);
            }
            _ => {}
        }
    }
}

fn collect_radio_receiver_comment(raw: &str, receivers: &mut BTreeMap<String, RadioTableReceiver>) {
    for line in raw.lines() {
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
}

fn radio_receiver_marker(line: &str, marker: &str) -> Option<String> {
    super::org_values::rows("radio-marker", &[line, marker])
        .pop()
        .map(|row| {
            let [value]: [String; 1] = row.try_into().expect("native receiver marker arity");
            value
        })
}
