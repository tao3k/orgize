//! Progress and statistics-cookie lint rules.

use crate::ast::{
    Checkbox, Element, ElementData, ListItem, Object, ObjectData, ParsedAnnotation, ParsedAst,
    ProgressCheckboxSummary, ProgressTodoSummary, Section, TodoState,
};

use super::lint_model::{LintFinding, LintSeverity, location_for_offsets};

pub(crate) fn progress_findings(document: &ParsedAst, source: &str) -> Vec<LintFinding> {
    let progress_records = document.progress_stats_records();
    let mut findings = Vec::new();
    for section in &document.sections {
        push_section_progress_findings(section, &progress_records, source, &mut findings);
    }
    findings
}

fn push_section_progress_findings(
    section: &Section<ParsedAnnotation>,
    progress_records: &[crate::ast::ProgressStatsRecord],
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    if let Some(record) = progress_records
        .iter()
        .find(|record| record.source.range_start == u32::from(section.ann.range.start()))
    {
        push_title_cookie_findings(section, record, source, findings);
    }
    push_list_item_cookie_findings(&section.children, source, &cookie_data(section), findings);
    for child in &section.subsections {
        push_section_progress_findings(child, progress_records, source, findings);
    }
}

fn push_title_cookie_findings(
    section: &Section<ParsedAnnotation>,
    record: &crate::ast::ProgressStatsRecord,
    source: &str,
    findings: &mut Vec<LintFinding>,
) {
    let cookies = statistic_cookies_from_objects(&section.title, None);
    if cookies.is_empty() {
        return;
    }

    let cookie_data = cookie_data(section);
    let domain = progress_domain(&cookie_data, record);
    let recursive = cookie_data
        .as_deref()
        .is_some_and(|value| value.contains("recursive"));

    for cookie in cookies {
        let Some(domain) = domain else {
            findings.push(ambiguous_cookie_finding(&cookie, source));
            continue;
        };
        if let Some(expected) = expected_cookie(domain, recursive, section, record, cookie.kind)
            && cookie.raw != expected.raw
        {
            findings.push(stale_cookie_finding(&cookie, domain, &expected.raw, source));
        }
    }
}

#[derive(Clone, Debug)]
struct StatisticCookie {
    raw: String,
    kind: CookieKind,
    range_start: usize,
    range_end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CookieKind {
    Fraction,
    Percent,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
enum ProgressDomain {
    Todo,
    Checkbox,
    Direct,
}

impl ProgressDomain {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Checkbox => "checkbox",
            Self::Direct => "direct",
        }
    }
}

struct ExpectedCookie {
    raw: String,
}

fn progress_domain(
    cookie_data: &Option<String>,
    record: &crate::ast::ProgressStatsRecord,
) -> Option<ProgressDomain> {
    if let Some(cookie_data) = cookie_data {
        if cookie_data.contains("direct") {
            return Some(ProgressDomain::Direct);
        }
        if cookie_data.contains("todo") {
            return Some(ProgressDomain::Todo);
        }
        if cookie_data.contains("checkbox") {
            return Some(ProgressDomain::Checkbox);
        }
    }
    match (
        record.descendant_todos.total > 0,
        record.checkboxes.total > 0,
    ) {
        (true, false) => Some(ProgressDomain::Todo),
        (false, true) => Some(ProgressDomain::Checkbox),
        (false, false) => Some(ProgressDomain::Todo),
        (true, true) => None,
    }
}

fn push_list_item_cookie_findings(
    elements: &[Element<ParsedAnnotation>],
    source: &str,
    cookie_data: &Option<String>,
    findings: &mut Vec<LintFinding>,
) {
    for element in elements {
        push_list_item_cookie_findings_from_element(element, source, cookie_data, findings);
    }
}

fn push_list_item_cookie_findings_from_element(
    element: &Element<ParsedAnnotation>,
    source: &str,
    cookie_data: &Option<String>,
    findings: &mut Vec<LintFinding>,
) {
    match &element.data {
        ElementData::List(list) => {
            for item in &list.items {
                push_one_list_item_cookie_findings(item, source, cookie_data, findings);
                push_list_item_cookie_findings(&item.children, source, cookie_data, findings);
            }
        }
        ElementData::Drawer(drawer) => {
            push_list_item_cookie_findings(&drawer.children, source, cookie_data, findings)
        }
        ElementData::Block(block) => {
            push_list_item_cookie_findings(&block.children, source, cookie_data, findings)
        }
        ElementData::FootnoteDef(footnote) => {
            push_list_item_cookie_findings(&footnote.children, source, cookie_data, findings);
        }
        ElementData::Inlinetask(task) => {
            push_list_item_cookie_findings(&task.children, source, cookie_data, findings);
        }
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

fn push_one_list_item_cookie_findings(
    item: &ListItem<ParsedAnnotation>,
    source: &str,
    cookie_data: &Option<String>,
    findings: &mut Vec<LintFinding>,
) {
    if cookie_data
        .as_deref()
        .is_some_and(|value| value.contains("todo"))
    {
        return;
    }

    let cookies = list_item_statistic_cookies(item);
    if cookies.is_empty() {
        return;
    }

    let recursive = cookie_data
        .as_deref()
        .is_some_and(|value| value.contains("recursive"));
    let summary = list_item_checkbox_summary(item, recursive);
    for cookie in &cookies {
        let Some(expected) = expected_checkbox_cookie(summary, cookie.kind) else {
            continue;
        };
        if cookie.raw != expected.raw {
            findings.push(stale_cookie_finding(
                cookie,
                ProgressDomain::Checkbox,
                &expected.raw,
                source,
            ));
        }
    }
}

fn list_item_statistic_cookies(item: &ListItem<ParsedAnnotation>) -> Vec<StatisticCookie> {
    let first_line = item.ann.start.line;
    let mut cookies = statistic_cookies_from_objects(&item.tag, Some(first_line));
    for element in &item.children {
        if let ElementData::Paragraph(objects) = &element.data {
            cookies.extend(statistic_cookies_from_objects(objects, Some(first_line)));
        }
    }
    cookies
}

fn list_item_checkbox_summary(
    item: &ListItem<ParsedAnnotation>,
    recursive: bool,
) -> ProgressCheckboxSummary {
    let mut summary = ProgressCheckboxSummary::default();
    for child in direct_child_items(item) {
        add_list_item_checkbox(child, recursive, &mut summary);
    }
    summary
}

fn add_list_item_checkbox(
    item: &ListItem<ParsedAnnotation>,
    recursive: bool,
    summary: &mut ProgressCheckboxSummary,
) {
    if let Some(checkbox) = item.checkbox {
        add_checkbox_state(checkbox, summary);
    }

    if recursive {
        collect_descendant_list_item_checkboxes(item, summary);
    }
}

fn add_checkbox_state(checkbox: Checkbox, summary: &mut ProgressCheckboxSummary) {
    summary.total += 1;
    match checkbox {
        Checkbox::On => summary.checked += 1,
        Checkbox::Off => summary.unchecked += 1,
        Checkbox::Trans => summary.partial += 1,
    }
}

fn collect_descendant_list_item_checkboxes(
    item: &ListItem<ParsedAnnotation>,
    summary: &mut ProgressCheckboxSummary,
) {
    for child in direct_child_items(item) {
        add_list_item_checkbox(child, true, summary);
    }
}

fn direct_child_items<'a>(
    item: &'a ListItem<ParsedAnnotation>,
) -> impl Iterator<Item = &'a ListItem<ParsedAnnotation>> + 'a {
    item.children
        .iter()
        .filter_map(list_items_from_element)
        .flat_map(|items| items.iter())
}

fn list_items_from_element(
    element: &Element<ParsedAnnotation>,
) -> Option<&[ListItem<ParsedAnnotation>]> {
    match &element.data {
        ElementData::List(list) => Some(&list.items),
        _ => None,
    }
}

fn expected_cookie(
    domain: ProgressDomain,
    recursive: bool,
    section: &Section<ParsedAnnotation>,
    record: &crate::ast::ProgressStatsRecord,
    kind: CookieKind,
) -> Option<ExpectedCookie> {
    let (done, total) = match domain {
        ProgressDomain::Todo => {
            let summary = if recursive {
                record.descendant_todos
            } else {
                direct_todo_summary(section)
            };
            (summary.done, summary.total)
        }
        ProgressDomain::Checkbox => {
            let summary = if recursive {
                record.checkboxes
            } else {
                direct_checkbox_summary(section)
            };
            (summary.checked, summary.total)
        }
        ProgressDomain::Direct => {
            let todo = if recursive {
                record.descendant_todos
            } else {
                direct_todo_summary(section)
            };
            let checkbox = if recursive {
                record.checkboxes
            } else {
                direct_checkbox_summary(section)
            };
            (todo.done + checkbox.checked, todo.total + checkbox.total)
        }
    };
    match kind {
        CookieKind::Fraction => Some(ExpectedCookie {
            raw: format!("[{done}/{total}]"),
        }),
        CookieKind::Percent => Some(ExpectedCookie {
            raw: format!("[{}%]", percent_cookie(done, total)),
        }),
        CookieKind::Unknown => None,
    }
}

fn expected_checkbox_cookie(
    summary: ProgressCheckboxSummary,
    kind: CookieKind,
) -> Option<ExpectedCookie> {
    match kind {
        CookieKind::Fraction => Some(ExpectedCookie {
            raw: format!("[{}/{}]", summary.checked, summary.total),
        }),
        CookieKind::Percent => Some(ExpectedCookie {
            raw: format!("[{}%]", percent_cookie(summary.checked, summary.total)),
        }),
        CookieKind::Unknown => None,
    }
}

fn direct_todo_summary(section: &Section<ParsedAnnotation>) -> ProgressTodoSummary {
    let mut summary = ProgressTodoSummary::default();
    for child in &section.subsections {
        let Some(todo) = &child.todo else {
            continue;
        };
        summary.total += 1;
        match todo.state {
            TodoState::Done => summary.done += 1,
            TodoState::Todo => summary.open += 1,
        }
    }
    summary
}

fn direct_checkbox_summary(section: &Section<ParsedAnnotation>) -> ProgressCheckboxSummary {
    let mut summary = ProgressCheckboxSummary::default();
    for element in &section.children {
        add_direct_element_checkboxes(element, &mut summary);
    }
    summary
}

fn add_direct_element_checkboxes(
    element: &Element<ParsedAnnotation>,
    summary: &mut ProgressCheckboxSummary,
) {
    if let ElementData::List(list) = &element.data {
        for item in &list.items {
            if let Some(checkbox) = item.checkbox {
                add_checkbox_state(checkbox, summary);
            }
        }
    }
}

fn cookie_data(section: &Section<ParsedAnnotation>) -> Option<String> {
    section
        .effective_properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case("COOKIE_DATA"))
        .map(|property| property.value.to_ascii_lowercase())
}

fn ambiguous_cookie_finding(cookie: &StatisticCookie, source: &str) -> LintFinding {
    LintFinding {
        code: "ORG027",
        severity: LintSeverity::Warning,
        message: "statistics cookie is ambiguous because this heading has both TODO children and checkboxes; set COOKIE_DATA to todo, checkbox, or direct".to_string(),
        location: location_for_offsets(source, cookie.range_start, cookie.range_end),
    }
}

fn stale_cookie_finding(
    cookie: &StatisticCookie,
    domain: ProgressDomain,
    expected: &str,
    source: &str,
) -> LintFinding {
    LintFinding {
        code: "ORG028",
        severity: LintSeverity::Warning,
        message: format!(
            "statistics cookie `{}` is stale for {} progress; expected `{expected}`",
            cookie.raw,
            domain.as_str()
        ),
        location: location_for_offsets(source, cookie.range_start, cookie.range_end),
    }
}

fn statistic_cookies_from_objects(
    objects: &[Object<ParsedAnnotation>],
    required_line: Option<usize>,
) -> Vec<StatisticCookie> {
    let mut cookies = Vec::new();
    collect_statistic_cookies_from_objects(objects, required_line, &mut cookies);
    cookies
}

fn collect_statistic_cookies_from_objects(
    objects: &[Object<ParsedAnnotation>],
    required_line: Option<usize>,
    cookies: &mut Vec<StatisticCookie>,
) {
    for object in objects {
        match &object.data {
            ObjectData::StatisticCookie(raw)
                if required_line.is_none_or(|line| object.ann.start.line == line) =>
            {
                let kind = statistic_cookie_kind(raw);
                cookies.push(StatisticCookie {
                    raw: raw.to_string(),
                    kind,
                    range_start: object.ann.range.start().into(),
                    range_end: object.ann.range.end().into(),
                });
            }
            ObjectData::Markup { children, .. } => {
                collect_statistic_cookies_from_objects(children, required_line, cookies)
            }
            ObjectData::FootnoteRef { definition, .. } => {
                collect_statistic_cookies_from_objects(definition, required_line, cookies);
            }
            ObjectData::Citation(citation) => {
                collect_statistic_cookies_from_objects(&citation.prefix, required_line, cookies);
                collect_statistic_cookies_from_objects(&citation.suffix, required_line, cookies);
                for reference in &citation.references {
                    collect_statistic_cookies_from_objects(
                        &reference.prefix,
                        required_line,
                        cookies,
                    );
                    collect_statistic_cookies_from_objects(
                        &reference.suffix,
                        required_line,
                        cookies,
                    );
                }
            }
            ObjectData::Cloze { text, .. } => {
                collect_statistic_cookies_from_objects(text, required_line, cookies)
            }
            ObjectData::Plain(_)
            | ObjectData::StatisticCookie(_)
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

fn statistic_cookie_kind(raw: &str) -> CookieKind {
    let value = raw.trim().trim_start_matches('[').trim_end_matches(']');
    if value.strip_suffix('%').and_then(parse_percent).is_some() {
        return CookieKind::Percent;
    }
    if parse_fraction_cookie(value).is_some() {
        return CookieKind::Fraction;
    }
    CookieKind::Unknown
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

fn percent_cookie(done: u32, total: u32) -> u8 {
    let percent = ((done as f64 * 100.0) / total.max(1) as f64).floor() as u8;
    if percent == 0 && done > 0 { 1 } else { percent }
}
