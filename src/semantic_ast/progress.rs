//! Progress, effort, and dependency rollups over parsed Org sections.

use super::{
    Checkbox, Document, Element, ElementData, Object, ObjectData, ParsedAnnotation,
    ProgressCheckboxSummary, ProgressEffortSummary, ProgressStatisticCookie,
    ProgressStatisticCookieKind, ProgressStatsRecord, ProgressTodoState, ProgressTodoSummary,
    Property, Section, SectionIndexSource, TaskDependencyKind, TaskDependencyRecord, TodoState,
};

impl Document<ParsedAnnotation> {
    /// Projects section-local progress, effort, and dependency rollups.
    ///
    /// This is a read-only semantic projection. It does not update statistics
    /// cookies, enforce dependencies, or mutate Org source.
    pub fn progress_stats_records(&self) -> Vec<ProgressStatsRecord> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_progress_record(section, Vec::new(), &mut records);
        }
        records.sort_by_key(|record| record.source.range_start);
        records
    }
}

#[derive(Clone, Debug, Default)]
struct ProgressRollup {
    todos: ProgressTodoSummary,
    checkboxes: ProgressCheckboxSummary,
    effort_seconds: u64,
}

fn collect_progress_record(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    records: &mut Vec<ProgressStatsRecord>,
) -> ProgressRollup {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path;
    outline_path.push(title.clone());

    let local = local_progress(section);
    let mut subtree = local.rollup.clone();
    let mut descendant_todos = ProgressTodoSummary::default();

    for child in &section.subsections {
        let child_rollup = collect_progress_record(child, outline_path.clone(), records);
        merge_todos(&mut descendant_todos, child_rollup.todos);
        merge_todos(&mut subtree.todos, child_rollup.todos);
        merge_checkboxes(&mut subtree.checkboxes, child_rollup.checkboxes);
        subtree.effort_seconds += child_rollup.effort_seconds;
    }

    let mut returned_rollup = subtree.clone();
    add_todo(
        &mut returned_rollup.todos,
        section.todo.as_ref().map(|todo| todo.state),
    );

    records.push(progress_record(
        section,
        outline_path,
        title,
        local,
        subtree,
        descendant_todos,
    ));
    returned_rollup
}

#[derive(Clone, Debug)]
struct LocalProgress {
    rollup: ProgressRollup,
    effort: Option<super::OrgDuration>,
    statistic_cookies: Vec<ProgressStatisticCookie>,
}

fn local_progress(section: &Section<ParsedAnnotation>) -> LocalProgress {
    let effort = local_effort(&section.properties);
    let mut statistic_cookies = Vec::new();
    collect_statistic_cookies_from_objects(&section.title, &mut statistic_cookies);
    collect_statistic_cookies_from_elements(&section.children, &mut statistic_cookies);

    let checkboxes = checkbox_summary_from_elements(&section.children);
    LocalProgress {
        rollup: ProgressRollup {
            todos: ProgressTodoSummary::default(),
            checkboxes,
            effort_seconds: effort.as_ref().map_or(0, |duration| duration.total_seconds),
        },
        effort,
        statistic_cookies,
    }
}

fn progress_record(
    section: &Section<ParsedAnnotation>,
    outline_path: Vec<String>,
    title: String,
    local: LocalProgress,
    subtree: ProgressRollup,
    descendant_todos: ProgressTodoSummary,
) -> ProgressStatsRecord {
    let dependencies = dependency_records(section, descendant_todos, subtree.checkboxes);
    ProgressStatsRecord {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path,
        level: section.level,
        title,
        todo: progress_todo_state(section.todo.as_ref().map(|todo| todo.state)),
        descendant_todos,
        checkboxes: subtree.checkboxes,
        statistic_cookies: local.statistic_cookies,
        effort: ProgressEffortSummary {
            local: local.effort,
            subtree_total_seconds: subtree.effort_seconds,
        },
        dependencies,
    }
}

fn dependency_records(
    section: &Section<ParsedAnnotation>,
    descendant_todos: ProgressTodoSummary,
    checkboxes: ProgressCheckboxSummary,
) -> Vec<TaskDependencyRecord> {
    let mut dependencies = Vec::new();
    push_count_dependency(
        &mut dependencies,
        section,
        TaskDependencyKind::OpenDescendantTodo,
        descendant_todos.open,
        "open descendant TODO entries can block parent task completion",
    );
    push_count_dependency(
        &mut dependencies,
        section,
        TaskDependencyKind::OpenCheckbox,
        checkboxes.unresolved(),
        "unchecked or partial checkboxes can block completion evidence",
    );
    if has_ordered_property(section) {
        push_count_dependency(
            &mut dependencies,
            section,
            TaskDependencyKind::OrderedProperty,
            1,
            "ORDERED property marks sibling order as dependency evidence",
        );
    }
    dependencies
}

fn push_count_dependency(
    dependencies: &mut Vec<TaskDependencyRecord>,
    section: &Section<ParsedAnnotation>,
    kind: TaskDependencyKind,
    count: u32,
    message: &'static str,
) {
    if count == 0 {
        return;
    }
    dependencies.push(TaskDependencyRecord {
        source: SectionIndexSource::from_annotation(&section.ann),
        kind,
        count,
        message: message.to_string(),
    });
}

fn has_ordered_property(section: &Section<ParsedAnnotation>) -> bool {
    section.properties.iter().any(|property| {
        property.key.eq_ignore_ascii_case("ORDERED") && is_truthy_property_value(&property.value)
    })
}

fn is_truthy_property_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "t" | "true" | "yes" | "1"
    )
}

fn local_effort(properties: &[Property<ParsedAnnotation>]) -> Option<super::OrgDuration> {
    properties
        .iter()
        .find(|property| property.is_effort())
        .and_then(|property| property.parsed_duration().cloned())
}

fn checkbox_summary_from_elements(
    elements: &[Element<ParsedAnnotation>],
) -> ProgressCheckboxSummary {
    let mut summary = ProgressCheckboxSummary::default();
    collect_checkboxes_from_elements(elements, &mut summary);
    summary
}

fn collect_checkboxes_from_elements(
    elements: &[Element<ParsedAnnotation>],
    summary: &mut ProgressCheckboxSummary,
) {
    for element in elements {
        collect_checkboxes_from_element(element, summary);
    }
}

fn collect_checkboxes_from_element(
    element: &Element<ParsedAnnotation>,
    summary: &mut ProgressCheckboxSummary,
) {
    match &element.data {
        ElementData::List(list) => {
            for item in &list.items {
                collect_checkboxes_from_item(item.checkbox, &item.children, summary);
            }
        }
        ElementData::Drawer(drawer) => collect_checkboxes_from_elements(&drawer.children, summary),
        ElementData::Block(block) => collect_checkboxes_from_elements(&block.children, summary),
        ElementData::FootnoteDef(footnote) => {
            collect_checkboxes_from_elements(&footnote.children, summary);
        }
        ElementData::Inlinetask(task) => collect_checkboxes_from_elements(&task.children, summary),
        ElementData::Paragraph(_)
        | ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::Table(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::DiarySexp(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_checkboxes_from_item(
    checkbox: Option<Checkbox>,
    children: &[Element<ParsedAnnotation>],
    summary: &mut ProgressCheckboxSummary,
) {
    if let Some(checkbox) = checkbox {
        summary.total += 1;
        match checkbox {
            Checkbox::On => summary.checked += 1,
            Checkbox::Off => summary.unchecked += 1,
            Checkbox::Trans => summary.partial += 1,
        }
    }
    collect_checkboxes_from_elements(children, summary);
}

fn collect_statistic_cookies_from_elements(
    elements: &[Element<ParsedAnnotation>],
    cookies: &mut Vec<ProgressStatisticCookie>,
) {
    for element in elements {
        collect_statistic_cookies_from_element(element, cookies);
    }
}

fn collect_statistic_cookies_from_element(
    element: &Element<ParsedAnnotation>,
    cookies: &mut Vec<ProgressStatisticCookie>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => collect_statistic_cookies_from_objects(objects, cookies),
        ElementData::List(list) => {
            for item in &list.items {
                collect_statistic_cookies_from_objects(&item.tag, cookies);
                collect_statistic_cookies_from_elements(&item.children, cookies);
            }
        }
        ElementData::Drawer(drawer) => {
            collect_statistic_cookies_from_elements(&drawer.children, cookies)
        }
        ElementData::Table(table) => {
            for row in &table.rows {
                for cell in &row.cells {
                    collect_statistic_cookies_from_objects(&cell.objects, cookies);
                }
            }
        }
        ElementData::Block(block) => {
            collect_statistic_cookies_from_elements(&block.children, cookies)
        }
        ElementData::FootnoteDef(footnote) => {
            collect_statistic_cookies_from_elements(&footnote.children, cookies);
        }
        ElementData::Inlinetask(task) => {
            collect_statistic_cookies_from_objects(&task.title, cookies);
            collect_statistic_cookies_from_elements(&task.children, cookies);
        }
        ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::DiarySexp(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_statistic_cookies_from_objects(
    objects: &[Object<ParsedAnnotation>],
    cookies: &mut Vec<ProgressStatisticCookie>,
) {
    for object in objects {
        match &object.data {
            ObjectData::StatisticCookie(raw) => {
                cookies.push(statistic_cookie(raw, &object.ann));
            }
            ObjectData::Markup { children, .. } => {
                collect_statistic_cookies_from_objects(children, cookies)
            }
            ObjectData::FootnoteRef { definition, .. } => {
                collect_statistic_cookies_from_objects(definition, cookies);
            }
            ObjectData::Citation(citation) => {
                collect_statistic_cookies_from_objects(&citation.prefix, cookies);
                collect_statistic_cookies_from_objects(&citation.suffix, cookies);
                for reference in &citation.references {
                    collect_statistic_cookies_from_objects(&reference.prefix, cookies);
                    collect_statistic_cookies_from_objects(&reference.suffix, cookies);
                }
            }
            ObjectData::Cloze { text, .. } => collect_statistic_cookies_from_objects(text, cookies),
            ObjectData::Plain(_)
            | ObjectData::LineBreak
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Entity(_)
            | ObjectData::LatexFragment(_)
            | ObjectData::ExportSnippet { .. }
            | ObjectData::Timestamp(_)
            | ObjectData::InlineCall { .. }
            | ObjectData::InlineSrc { .. }
            | ObjectData::Link(_)
            | ObjectData::Target(_)
            | ObjectData::RadioTarget(_)
            | ObjectData::Macro { .. }
            | ObjectData::Unknown { .. } => {}
        }
    }
}

fn statistic_cookie(raw: &str, annotation: &ParsedAnnotation) -> ProgressStatisticCookie {
    let (kind, done, total, percent) = statistic_cookie_parts(raw);
    ProgressStatisticCookie {
        source: SectionIndexSource::from_annotation(annotation),
        raw: raw.to_string(),
        kind,
        done,
        total,
        percent,
    }
}

fn statistic_cookie_parts(
    raw: &str,
) -> (
    ProgressStatisticCookieKind,
    Option<u32>,
    Option<u32>,
    Option<u8>,
) {
    let value = raw.trim().trim_start_matches('[').trim_end_matches(']');
    if let Some(percent) = value.strip_suffix('%').and_then(parse_percent) {
        return (
            ProgressStatisticCookieKind::Percent,
            None,
            None,
            Some(percent),
        );
    }
    if let Some((done, total)) = parse_fraction_cookie(value) {
        return (
            ProgressStatisticCookieKind::Fraction,
            Some(done),
            Some(total),
            None,
        );
    }
    (ProgressStatisticCookieKind::Unknown, None, None, None)
}

fn parse_fraction_cookie(value: &str) -> Option<(u32, u32)> {
    let (done, total) = value.split_once('/')?;
    Some((done.trim().parse().ok()?, total.trim().parse().ok()?))
}

fn parse_percent(value: &str) -> Option<u8> {
    value
        .trim()
        .parse::<u8>()
        .ok()
        .filter(|value| *value <= 100)
}

fn add_todo(summary: &mut ProgressTodoSummary, state: Option<TodoState>) {
    let Some(state) = state else {
        return;
    };
    summary.total += 1;
    match state {
        TodoState::Done => summary.done += 1,
        TodoState::Todo => summary.open += 1,
    }
}

fn merge_todos(summary: &mut ProgressTodoSummary, other: ProgressTodoSummary) {
    summary.total += other.total;
    summary.done += other.done;
    summary.open += other.open;
}

fn merge_checkboxes(summary: &mut ProgressCheckboxSummary, other: ProgressCheckboxSummary) {
    summary.total += other.total;
    summary.checked += other.checked;
    summary.unchecked += other.unchecked;
    summary.partial += other.partial;
}

fn progress_todo_state(state: Option<TodoState>) -> ProgressTodoState {
    match state {
        Some(TodoState::Todo) => ProgressTodoState::Todo,
        Some(TodoState::Done) => ProgressTodoState::Done,
        None => ProgressTodoState::None,
    }
}
