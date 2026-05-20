//! Clock and clocktable projections over parsed Org sections.

use super::block_metadata::parse_block_header_args;
use super::clock_table_properties::{clock_table_property_columns, clock_table_property_values};
use super::clock_table_time::{
    ClockTableWindowFilter, clipped_clock_seconds, clock_start_in_window, clock_table_time_window,
};
use super::dynamic_blocks::{ParsedDynamicBlockBegin, dynamic_block_begin};
use super::{AgendaMatchQuery, agenda_filter::section_matches_agenda_match};
use super::{
    BlockHeaderArg, BlockKind, Clock, ClockEffortStatus, ClockEffortSummary, ClockRollupRecord,
    ClockSummary, ClockTableMatchFilter, ClockTableParameter, ClockTablePlan,
    ClockTablePropertyColumns, ClockTableRow, ClockTableScope, ClockTableScopeKind,
    ClockTableWarning, ClockTableWarningKind, Document, Element, ElementData, OrgDuration,
    ParsedAnnotation, Property, Section, SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Projects section-local and subtree CLOCK totals with Effort comparison.
    ///
    /// This is a read-only semantic projection. It does not refresh CLOCK
    /// lines, update Effort properties, or rewrite clocktable dynamic blocks.
    pub fn clock_rollup_records(&self) -> Vec<ClockRollupRecord> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_clock_rollup_record(section, Vec::new(), &mut records);
        }
        records.sort_by_key(|record| record.source.range_start);
        records
    }

    /// Projects `#+BEGIN: clocktable` dynamic blocks into non-mutating plans.
    pub fn clock_table_plans(&self) -> Vec<ClockTablePlan> {
        let mut plans = Vec::new();
        collect_clock_table_plans_in_elements(
            &self.children,
            None,
            &[],
            &self.sections,
            &mut plans,
        );
        for section in &self.sections {
            collect_clock_table_plans_in_section(section, &[], &self.sections, &mut plans);
        }
        plans.sort_by_key(|plan| plan.source.range_start);
        plans
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Rollup {
    clock: ClockSummary,
    effort_seconds: u64,
}

fn collect_clock_rollup_record(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    records: &mut Vec<ClockRollupRecord>,
) -> Rollup {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path;
    outline_path.push(title.clone());

    let local_clock = clock_summary_from_elements(&section.children);
    let local_effort = local_effort(&section.properties);
    let mut subtree = Rollup {
        clock: local_clock,
        effort_seconds: local_effort
            .as_ref()
            .map_or(0, |duration| duration.total_seconds),
    };

    for child in &section.subsections {
        let child_rollup = collect_clock_rollup_record(child, outline_path.clone(), records);
        subtree.clock.merge(child_rollup.clock);
        subtree.effort_seconds += child_rollup.effort_seconds;
    }

    let delta_seconds = effort_delta_seconds(subtree.clock.total_seconds, subtree.effort_seconds);
    records.push(ClockRollupRecord {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path,
        level: section.level,
        title,
        local_clock,
        subtree_clock: subtree.clock,
        effort: ClockEffortSummary {
            local: local_effort,
            subtree_total_seconds: subtree.effort_seconds,
            delta_seconds,
            status: effort_status(subtree.clock.total_seconds, subtree.effort_seconds),
        },
    });

    subtree
}

fn clock_summary_from_elements(elements: &[Element<ParsedAnnotation>]) -> ClockSummary {
    clock_summary_from_elements_with_window(elements, None)
}

fn clock_summary_from_elements_with_window(
    elements: &[Element<ParsedAnnotation>],
    time_window: Option<&ClockTableWindowFilter>,
) -> ClockSummary {
    let mut summary = ClockSummary::default();
    collect_clock_summary_from_elements(elements, &mut summary, time_window);
    summary
}

fn collect_clock_summary_from_elements(
    elements: &[Element<ParsedAnnotation>],
    summary: &mut ClockSummary,
    time_window: Option<&ClockTableWindowFilter>,
) {
    for element in elements {
        collect_clock_summary_from_element(element, summary, time_window);
    }
}

fn collect_clock_summary_from_element(
    element: &Element<ParsedAnnotation>,
    summary: &mut ClockSummary,
    time_window: Option<&ClockTableWindowFilter>,
) {
    match &element.data {
        ElementData::Clock(clock) => add_clock(clock, summary, time_window),
        ElementData::Drawer(drawer) => {
            collect_clock_summary_from_elements(&drawer.children, summary, time_window)
        }
        ElementData::List(list) => {
            for item in &list.items {
                collect_clock_summary_from_elements(&item.children, summary, time_window);
            }
        }
        ElementData::Block(block) => {
            collect_clock_summary_from_elements(&block.children, summary, time_window)
        }
        ElementData::FootnoteDef(footnote) => {
            collect_clock_summary_from_elements(&footnote.children, summary, time_window);
        }
        ElementData::Inlinetask(task) => {
            collect_clock_summary_from_elements(&task.children, summary, time_window);
        }
        ElementData::Paragraph(_)
        | ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::Table(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn add_clock(
    clock: &Clock,
    summary: &mut ClockSummary,
    time_window: Option<&ClockTableWindowFilter>,
) {
    if let Some(time_window) = time_window {
        add_clock_in_window(clock, summary, time_window);
        return;
    }

    summary.entries += 1;
    if let Some(duration) = clock.parsed_duration.as_ref() {
        summary.closed_entries += 1;
        summary.total_seconds += duration.total_seconds;
    } else if clock.duration.is_some() {
        summary.unparsed_entries += 1;
    } else {
        summary.running_entries += 1;
    }
}

fn add_clock_in_window(
    clock: &Clock,
    summary: &mut ClockSummary,
    time_window: &ClockTableWindowFilter,
) {
    if let Some(duration) = clock.parsed_duration.as_ref() {
        match clipped_clock_seconds(clock, duration.total_seconds, time_window) {
            Some(0) => {}
            Some(seconds) => {
                summary.entries += 1;
                summary.closed_entries += 1;
                summary.total_seconds += seconds;
            }
            None => {
                summary.entries += 1;
                summary.unparsed_entries += 1;
            }
        }
        return;
    }

    if clock.duration.is_some() {
        if clock_start_in_window(clock, time_window).unwrap_or(true) {
            summary.entries += 1;
            summary.unparsed_entries += 1;
        }
        return;
    }

    if clock_start_in_window(clock, time_window).unwrap_or(true) {
        summary.entries += 1;
        summary.running_entries += 1;
    }
}

fn local_effort(properties: &[Property<ParsedAnnotation>]) -> Option<OrgDuration> {
    properties
        .iter()
        .find(|property| property.is_effort())
        .and_then(|property| property.parsed_duration().cloned())
}

fn effort_delta_seconds(clock_seconds: u64, effort_seconds: u64) -> i64 {
    let clock = clock_seconds.min(i64::MAX as u64) as i64;
    let effort = effort_seconds.min(i64::MAX as u64) as i64;
    clock - effort
}

fn effort_status(clock_seconds: u64, effort_seconds: u64) -> ClockEffortStatus {
    if effort_seconds == 0 {
        ClockEffortStatus::NoEffort
    } else if clock_seconds < effort_seconds {
        ClockEffortStatus::UnderEffort
    } else if clock_seconds == effort_seconds {
        ClockEffortStatus::OnEffort
    } else {
        ClockEffortStatus::OverEffort
    }
}

fn collect_clock_table_plans_in_section<'a>(
    section: &'a Section<ParsedAnnotation>,
    ancestors: &[&'a Section<ParsedAnnotation>],
    root_sections: &'a [Section<ParsedAnnotation>],
    plans: &mut Vec<ClockTablePlan>,
) {
    let mut stack = ancestors.to_vec();
    stack.push(section);
    collect_clock_table_plans_in_elements(
        &section.children,
        Some(section),
        &stack,
        root_sections,
        plans,
    );
    for child in &section.subsections {
        collect_clock_table_plans_in_section(child, &stack, root_sections, plans);
    }
}

fn collect_clock_table_plans_in_elements<'a>(
    elements: &[Element<ParsedAnnotation>],
    current_section: Option<&'a Section<ParsedAnnotation>>,
    section_stack: &[&'a Section<ParsedAnnotation>],
    root_sections: &'a [Section<ParsedAnnotation>],
    plans: &mut Vec<ClockTablePlan>,
) {
    for element in elements {
        if let Some(block) = clocktable_dynamic_block(element) {
            plans.push(clock_table_plan(
                element,
                block,
                current_section,
                section_stack,
                root_sections,
            ));
        }
        match &element.data {
            ElementData::Drawer(drawer) => collect_clock_table_plans_in_elements(
                &drawer.children,
                current_section,
                section_stack,
                root_sections,
                plans,
            ),
            ElementData::List(list) => {
                for item in &list.items {
                    collect_clock_table_plans_in_elements(
                        &item.children,
                        current_section,
                        section_stack,
                        root_sections,
                        plans,
                    );
                }
            }
            ElementData::Block(block) => collect_clock_table_plans_in_elements(
                &block.children,
                current_section,
                section_stack,
                root_sections,
                plans,
            ),
            ElementData::FootnoteDef(footnote) => collect_clock_table_plans_in_elements(
                &footnote.children,
                current_section,
                section_stack,
                root_sections,
                plans,
            ),
            ElementData::Inlinetask(task) => collect_clock_table_plans_in_elements(
                &task.children,
                current_section,
                section_stack,
                root_sections,
                plans,
            ),
            ElementData::Paragraph(_)
            | ElementData::Keyword(_)
            | ElementData::BabelCall(_)
            | ElementData::Clock(_)
            | ElementData::PropertyDrawer(_)
            | ElementData::Table(_)
            | ElementData::TableEl { .. }
            | ElementData::Comment(_)
            | ElementData::FixedWidth(_)
            | ElementData::Rule
            | ElementData::LatexEnvironment(_)
            | ElementData::Unknown { .. } => {}
        }
    }
}

fn clocktable_dynamic_block(
    element: &Element<ParsedAnnotation>,
) -> Option<ParsedDynamicBlockBegin> {
    let ElementData::Block(block) = &element.data else {
        return None;
    };
    if block.kind != BlockKind::Dynamic {
        return None;
    }
    let dynamic = dynamic_block_begin(&element.ann.raw)?;
    dynamic
        .name
        .eq_ignore_ascii_case("clocktable")
        .then_some(dynamic)
}

fn clock_table_plan<'a>(
    element: &Element<ParsedAnnotation>,
    block: ParsedDynamicBlockBegin,
    current_section: Option<&'a Section<ParsedAnnotation>>,
    section_stack: &[&'a Section<ParsedAnnotation>],
    root_sections: &'a [Section<ParsedAnnotation>],
) -> ClockTablePlan {
    let parameters = clock_table_parameters(&block.parameters);
    let scope = clock_table_scope(&parameters);
    let max_level = clock_table_max_level(&parameters);
    let tstart = parameter_value(&parameters, "tstart");
    let tend = parameter_value(&parameters, "tend");
    let (time_window, mut warnings) = clock_table_time_window(&parameters);
    let (match_filter, match_warnings) = clock_table_match_filter(&parameters);
    let (property_columns, property_warnings) = clock_table_property_columns(&parameters);
    warnings.extend(match_warnings);
    warnings.extend(property_warnings);
    warnings.extend(clock_table_warnings(&parameters, &scope));
    let scope_section = scope_section(current_section, section_stack, &scope);
    if scope_section.is_none()
        && !matches!(
            scope.kind,
            ClockTableScopeKind::File | ClockTableScopeKind::Nil
        )
    {
        warnings.push(ClockTableWarning {
            kind: ClockTableWarningKind::UnsupportedScope,
            message: "scope requires files or agenda context outside this parsed document"
                .to_string(),
        });
    }

    let rows = if clock_table_scope_is_document_local(&scope) {
        clock_table_rows(
            root_sections,
            scope_section,
            section_stack,
            max_level,
            time_window.as_ref(),
            match_filter.as_ref(),
            property_columns.as_ref(),
        )
    } else {
        Vec::new()
    };

    ClockTablePlan {
        source: SectionIndexSource::from_annotation(&element.ann),
        name: block.name,
        parameters,
        scope,
        max_level,
        tstart,
        tend,
        time_window: time_window.map(|time_window| time_window.window),
        match_filter: match_filter.as_ref().map(|filter| filter.filter.clone()),
        property_columns,
        rows,
        warnings,
    }
}

fn clock_table_parameters(parameters: &str) -> Vec<ClockTableParameter> {
    parse_block_header_args((!parameters.trim().is_empty()).then_some(parameters))
        .into_iter()
        .map(clock_table_parameter)
        .collect()
}

fn clock_table_parameter(parameter: BlockHeaderArg) -> ClockTableParameter {
    ClockTableParameter {
        key: parameter.key,
        value: parameter.value,
        raw: parameter.raw,
    }
}

fn clock_table_scope(parameters: &[ClockTableParameter]) -> ClockTableScope {
    let Some(value) = parameter_value(parameters, "scope") else {
        return ClockTableScope {
            kind: ClockTableScopeKind::File,
            value: Some("file".to_string()),
        };
    };
    let normalized = value
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase();
    let kind = match normalized.as_str() {
        "nil" => ClockTableScopeKind::Nil,
        "file" => ClockTableScopeKind::File,
        "subtree" => ClockTableScopeKind::Subtree,
        "tree" => ClockTableScopeKind::Tree,
        "agenda" => ClockTableScopeKind::Agenda,
        "agenda-with-archives" => ClockTableScopeKind::AgendaWithArchives,
        "file-with-archives" => ClockTableScopeKind::FileWithArchives,
        value if tree_level(value).is_some() => ClockTableScopeKind::TreeLevel,
        value if value.starts_with('(') || value.ends_with(')') => ClockTableScopeKind::External,
        value
            if value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_') =>
        {
            ClockTableScopeKind::Unknown
        }
        _ => ClockTableScopeKind::External,
    };
    ClockTableScope {
        kind,
        value: Some(value),
    }
}

fn clock_table_max_level(parameters: &[ClockTableParameter]) -> usize {
    parameter_value(parameters, "maxlevel")
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(2)
}

fn parameter_value(parameters: &[ClockTableParameter], key: &str) -> Option<String> {
    parameters
        .iter()
        .find(|parameter| parameter.key.eq_ignore_ascii_case(key))
        .and_then(|parameter| parameter.value.clone())
}

fn parameter_present(parameters: &[ClockTableParameter], key: &str) -> bool {
    parameters
        .iter()
        .any(|parameter| parameter.key.eq_ignore_ascii_case(key))
}

#[derive(Clone, Debug)]
struct ClockTableMatchRuntime {
    filter: ClockTableMatchFilter,
    query: AgendaMatchQuery,
}

#[derive(Clone, Copy, Debug)]
struct ClockTableRowOptions<'a> {
    max_level: usize,
    time_window: Option<&'a ClockTableWindowFilter>,
    match_filter: Option<&'a ClockTableMatchRuntime>,
    property_columns: Option<&'a ClockTablePropertyColumns>,
}

fn clock_table_match_filter(
    parameters: &[ClockTableParameter],
) -> (Option<ClockTableMatchRuntime>, Vec<ClockTableWarning>) {
    if !parameter_present(parameters, "match") {
        return (None, Vec::new());
    }

    let Some(raw) = parameter_value(parameters, "match") else {
        return (
            None,
            vec![clock_table_match_warning(
                "match parameter has no expression",
            )],
        );
    };
    let expression = normalized_parameter_value(&raw);
    match AgendaMatchQuery::parse(&expression) {
        Ok(query) => (
            Some(ClockTableMatchRuntime {
                filter: ClockTableMatchFilter { expression },
                query,
            }),
            Vec::new(),
        ),
        Err(error) => (
            None,
            vec![clock_table_match_warning(format!(
                "match parameter is preserved but not applied: {error}"
            ))],
        ),
    }
}

fn normalized_parameter_value(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn clock_table_match_warning(message: impl Into<String>) -> ClockTableWarning {
    ClockTableWarning {
        kind: ClockTableWarningKind::MatchPreserved,
        message: message.into(),
    }
}

fn clock_table_warnings(
    parameters: &[ClockTableParameter],
    scope: &ClockTableScope,
) -> Vec<ClockTableWarning> {
    let mut warnings = Vec::new();
    if parameter_value(parameters, "step").is_some() {
        warnings.push(ClockTableWarning {
            kind: ClockTableWarningKind::StepPreserved,
            message: "step parameter is preserved but split clocktables are not generated"
                .to_string(),
        });
    }
    if matches!(
        scope.kind,
        ClockTableScopeKind::Agenda
            | ClockTableScopeKind::AgendaWithArchives
            | ClockTableScopeKind::FileWithArchives
            | ClockTableScopeKind::External
            | ClockTableScopeKind::Unknown
    ) {
        warnings.push(ClockTableWarning {
            kind: ClockTableWarningKind::UnsupportedScope,
            message: "scope is preserved; only file, subtree, tree, and treeN are resolved inside one parsed document"
                .to_string(),
        });
    }
    warnings
}

fn scope_section<'a>(
    current_section: Option<&'a Section<ParsedAnnotation>>,
    section_stack: &[&'a Section<ParsedAnnotation>],
    scope: &ClockTableScope,
) -> Option<&'a Section<ParsedAnnotation>> {
    match scope.kind {
        ClockTableScopeKind::Subtree => current_section,
        ClockTableScopeKind::Tree => section_stack.first().copied().or(current_section),
        ClockTableScopeKind::TreeLevel => scope
            .value
            .as_deref()
            .and_then(tree_level)
            .and_then(|level| {
                section_stack
                    .iter()
                    .rev()
                    .copied()
                    .find(|section| section.level <= level)
            })
            .or_else(|| section_stack.first().copied())
            .or(current_section),
        ClockTableScopeKind::File | ClockTableScopeKind::Nil => None,
        ClockTableScopeKind::Agenda
        | ClockTableScopeKind::AgendaWithArchives
        | ClockTableScopeKind::FileWithArchives
        | ClockTableScopeKind::External
        | ClockTableScopeKind::Unknown => None,
    }
}

fn clock_table_scope_is_document_local(scope: &ClockTableScope) -> bool {
    matches!(
        scope.kind,
        ClockTableScopeKind::File
            | ClockTableScopeKind::Nil
            | ClockTableScopeKind::Subtree
            | ClockTableScopeKind::Tree
            | ClockTableScopeKind::TreeLevel
    )
}

fn tree_level(value: &str) -> Option<usize> {
    value
        .strip_prefix("tree")
        .filter(|level| !level.is_empty())
        .and_then(|level| level.parse().ok())
}

fn clock_table_rows<'a>(
    root_sections: &'a [Section<ParsedAnnotation>],
    scope_section: Option<&Section<ParsedAnnotation>>,
    section_stack: &[&'a Section<ParsedAnnotation>],
    max_level: usize,
    time_window: Option<&ClockTableWindowFilter>,
    match_filter: Option<&ClockTableMatchRuntime>,
    property_columns: Option<&ClockTablePropertyColumns>,
) -> Vec<ClockTableRow> {
    let mut rows = Vec::new();
    let options = ClockTableRowOptions {
        max_level,
        time_window,
        match_filter,
        property_columns,
    };
    if let Some(section) = scope_section {
        let prefix = outline_prefix_before_scope(section, section_stack);
        let scope_depth = prefix.len() + 1;
        collect_clock_table_row_from_section(section, prefix, scope_depth, options, &mut rows);
    } else {
        for section in root_sections {
            collect_clock_table_row_from_section(section, Vec::new(), 0, options, &mut rows);
        }
    }

    rows.sort_by_key(|row| row.source.range_start);
    rows
}

fn collect_clock_table_row_from_section(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    scope_depth: usize,
    options: ClockTableRowOptions<'_>,
    rows: &mut Vec<ClockTableRow>,
) -> Rollup {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path;
    outline_path.push(title.clone());

    let local_matches = options
        .match_filter
        .map(|filter| section_matches_agenda_match(section, None, None, &filter.query))
        .unwrap_or(true);
    let local_clock = if local_matches {
        clock_summary_from_elements_with_window(&section.children, options.time_window)
    } else {
        ClockSummary::default()
    };
    let local_effort = local_matches
        .then(|| local_effort(&section.properties))
        .flatten();
    let mut subtree = Rollup {
        clock: local_clock,
        effort_seconds: local_effort
            .as_ref()
            .map_or(0, |duration| duration.total_seconds),
    };

    for child in &section.subsections {
        let child_rollup = collect_clock_table_row_from_section(
            child,
            outline_path.clone(),
            scope_depth,
            options,
            rows,
        );
        subtree.clock.merge(child_rollup.clock);
        subtree.effort_seconds += child_rollup.effort_seconds;
    }

    let table_level = scoped_table_level(outline_path.len(), scope_depth);
    if table_level <= options.max_level
        && (subtree.clock.total_seconds > 0 || subtree.effort_seconds > 0)
    {
        let delta_seconds =
            effort_delta_seconds(subtree.clock.total_seconds, subtree.effort_seconds);
        rows.push(ClockTableRow {
            source: SectionIndexSource::from_annotation(&section.ann),
            outline_path,
            level: section.level,
            table_level,
            title,
            clock: subtree.clock,
            effort_total_seconds: subtree.effort_seconds,
            effort_delta_seconds: delta_seconds,
            effort_status: effort_status(subtree.clock.total_seconds, subtree.effort_seconds),
            property_values: options
                .property_columns
                .map(|columns| clock_table_property_values(section, columns))
                .unwrap_or_default(),
        });
    }

    subtree
}

fn outline_prefix_before_scope(
    scope_section: &Section<ParsedAnnotation>,
    section_stack: &[&Section<ParsedAnnotation>],
) -> Vec<String> {
    let mut prefix = Vec::new();
    for section in section_stack {
        if std::ptr::eq(*section, scope_section) {
            break;
        }
        prefix.push(section.raw_title.trim_end().to_string());
    }
    prefix
}

fn scoped_table_level(outline_path_len: usize, scope_depth: usize) -> usize {
    if scope_depth == 0 {
        outline_path_len
    } else {
        outline_path_len.saturating_sub(scope_depth) + 1
    }
}
