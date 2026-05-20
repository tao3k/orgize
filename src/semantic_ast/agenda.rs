//! Semantic agenda projection from headline planning timestamps.

use super::agenda_filter::section_matches_query;
use super::agenda_model::{
    AgendaCategory, AgendaDate, AgendaDeadlineState, AgendaEntry, AgendaEntryKind,
    AgendaOccurrence, AgendaQuery, AgendaScheduleState, AgendaTime, scheduled_visible_start,
    warning_start,
};
use super::agenda_time::{HeadlineTimeSpec, headline_time};
use super::model::{
    Citation, CiteReference, Document, Element, ElementData, Link, ListItem, Object, ObjectData,
    Property, Section,
};
use super::timestamp_model::{Timestamp, TimestampKind};

impl<A: Clone> Document<A> {
    /// Projects headline planning timestamps into agenda rows.
    ///
    /// This is an opt-in semantic view: it does not mutate the parsed AST and it
    /// does not change the lossless syntax/export substrate.
    pub fn agenda_entries(&self, query: &AgendaQuery) -> Vec<AgendaEntry<A>> {
        let mut entries = Vec::new();
        let (start, end) = query.bounds();
        let document_category = document_category(self);

        for section in &self.sections {
            collect_section(
                section,
                AgendaCollectContext { query, start, end },
                document_category.clone(),
                &mut entries,
            );
        }

        entries.sort_by(|left, right| {
            (
                left.display_date,
                left.time,
                kind_order(left.kind),
                left.level,
                left.raw_title.as_str(),
                left.target_date,
            )
                .cmp(&(
                    right.display_date,
                    right.time,
                    kind_order(right.kind),
                    right.level,
                    right.raw_title.as_str(),
                    right.target_date,
                ))
        });
        entries
    }
}

#[derive(Clone, Copy)]
struct AgendaCollectContext<'a> {
    query: &'a AgendaQuery,
    start: AgendaDate,
    end: AgendaDate,
}

fn collect_section<A: Clone>(
    section: &Section<A>,
    context: AgendaCollectContext<'_>,
    inherited_category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    let category = section_category(section).or(inherited_category);

    if section_matches_query(section, context.query, category.as_ref()) {
        if context.query.include_scheduled {
            collect_timestamp(
                section,
                AgendaEntryKind::Scheduled,
                section.planning.scheduled.as_ref(),
                context,
                category.clone(),
                entries,
            );
        }

        if context.query.include_deadlines {
            collect_timestamp(
                section,
                AgendaEntryKind::Deadline,
                section.planning.deadline.as_ref(),
                context,
                category.clone(),
                entries,
            );
        }

        if context.query.include_closed {
            collect_timestamp(
                section,
                AgendaEntryKind::Closed,
                section.planning.closed.as_ref(),
                context,
                category.clone(),
                entries,
            );
        }

        if context.query.include_timestamps {
            collect_active_timestamps_in_objects(
                section,
                &section.title,
                context,
                category.clone(),
                entries,
            );
            collect_active_timestamps_in_elements(
                section,
                &section.children,
                context,
                category.clone(),
                entries,
            );
        }
    }

    for subsection in &section.subsections {
        collect_section(subsection, context, category.clone(), entries);
    }
}

fn collect_timestamp<A: Clone>(
    section: &Section<A>,
    kind: AgendaEntryKind,
    timestamp: Option<&Timestamp>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    let Some(timestamp) = timestamp else {
        return;
    };
    let Some(moment) = &timestamp.start else {
        return;
    };

    let base_start = AgendaDate::from_moment(moment);
    let base_end = timestamp.end.as_ref().map(AgendaDate::from_moment);
    let time = AgendaTime::from_moment(moment);
    let end_time = timestamp.end.as_ref().and_then(AgendaTime::from_moment);
    let target_end = match kind {
        AgendaEntryKind::Deadline if context.query.include_deadline_warnings => timestamp
            .warning
            .as_ref()
            .and_then(|warning| context.end.add_interval(warning.value as i32, warning.unit))
            .unwrap_or(context.end),
        _ => context.end,
    };
    let occurrences = occurrence_spans(
        timestamp,
        base_start,
        base_end,
        target_end,
        context.query.expand_repeaters,
    );

    for span in occurrences {
        let seed = EntrySeed {
            ann: section.ann.clone(),
            section,
            kind,
            timestamp,
            target_date: span.start,
            target_end_date: span.end,
            time,
            end_time,
            headline_time: headline_time(section, context.query),
            category: category.clone(),
            occurrence: span.occurrence,
        };
        match kind {
            AgendaEntryKind::Deadline => collect_deadline_entries(seed, context, entries),
            AgendaEntryKind::Scheduled => {
                collect_scheduled_entries(&seed, context.start, context.end, entries);
            }
            AgendaEntryKind::Timestamp | AgendaEntryKind::Diary | AgendaEntryKind::Closed => {
                collect_span_entries(&seed, context.start, context.end, entries);
            }
        }
    }
}

fn collect_active_timestamps_in_elements<A: Clone>(
    section: &Section<A>,
    elements: &[Element<A>],
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    for element in elements {
        collect_active_timestamps_in_element(section, element, context, category.clone(), entries);
    }
}

fn collect_active_timestamps_in_element<A: Clone>(
    section: &Section<A>,
    element: &Element<A>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => {
            collect_active_timestamps_in_objects(section, objects, context, category, entries);
        }
        ElementData::Keyword(_) | ElementData::BabelCall(_) => {}
        ElementData::Clock(clock) => {
            if let Some(timestamp) = &clock.value {
                collect_active_timestamp(
                    section,
                    &element.ann,
                    timestamp,
                    context,
                    category,
                    entries,
                );
            }
        }
        ElementData::Drawer(drawer) => {
            collect_active_timestamps_in_elements(
                section,
                &drawer.children,
                context,
                category,
                entries,
            );
        }
        ElementData::PropertyDrawer(_) => {}
        ElementData::List(list) => {
            for item in &list.items {
                collect_active_timestamps_in_list_item(
                    section,
                    item,
                    context,
                    category.clone(),
                    entries,
                );
            }
        }
        ElementData::Table(table) => {
            for row in &table.rows {
                for cell in &row.cells {
                    collect_active_timestamps_in_objects(
                        section,
                        &cell.objects,
                        context,
                        category.clone(),
                        entries,
                    );
                }
            }
        }
        ElementData::Block(block) => {
            collect_active_timestamps_in_elements(
                section,
                &block.children,
                context,
                category,
                entries,
            );
        }
        ElementData::FootnoteDef(footnote) => {
            collect_active_timestamps_in_elements(
                section,
                &footnote.children,
                context,
                category,
                entries,
            );
        }
        ElementData::Inlinetask(task) => {
            collect_active_timestamps_in_elements(
                section,
                &task.children,
                context,
                category,
                entries,
            );
        }
        ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_active_timestamps_in_list_item<A: Clone>(
    section: &Section<A>,
    item: &ListItem<A>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    collect_active_timestamps_in_objects(section, &item.tag, context, category.clone(), entries);
    collect_active_timestamps_in_elements(section, &item.children, context, category, entries);
}

fn collect_active_timestamps_in_objects<A: Clone>(
    section: &Section<A>,
    objects: &[Object<A>],
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    for object in objects {
        collect_active_timestamps_in_object(section, object, context, category.clone(), entries);
    }
}

fn collect_active_timestamps_in_object<A: Clone>(
    section: &Section<A>,
    object: &Object<A>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    match &object.data {
        ObjectData::Timestamp(timestamp) => {
            collect_active_timestamp(section, &object.ann, timestamp, context, category, entries);
        }
        ObjectData::Markup { children, .. } => {
            collect_active_timestamps_in_objects(section, children, context, category, entries);
        }
        ObjectData::FootnoteRef { definition, .. } => {
            collect_active_timestamps_in_objects(section, definition, context, category, entries);
        }
        ObjectData::Citation(citation) => {
            collect_active_timestamps_in_citation(section, citation, context, category, entries);
        }
        ObjectData::Cloze { text, .. } => {
            collect_active_timestamps_in_objects(section, text, context, category, entries);
        }
        ObjectData::Link(link) => {
            collect_active_timestamps_in_link(section, link, context, category, entries);
        }
        ObjectData::Plain(_)
        | ObjectData::LineBreak
        | ObjectData::Code(_)
        | ObjectData::Verbatim(_)
        | ObjectData::Entity(_)
        | ObjectData::LatexFragment(_)
        | ObjectData::ExportSnippet { .. }
        | ObjectData::InlineCall { .. }
        | ObjectData::InlineSrc { .. }
        | ObjectData::Target(_)
        | ObjectData::RadioTarget(_)
        | ObjectData::Macro { .. }
        | ObjectData::StatisticCookie(_)
        | ObjectData::Unknown { .. } => {}
    }
}

fn collect_active_timestamps_in_link<A: Clone>(
    section: &Section<A>,
    link: &Link<A>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    collect_active_timestamps_in_objects(section, &link.description, context, category, entries);
}

fn collect_active_timestamps_in_citation<A: Clone>(
    section: &Section<A>,
    citation: &Citation<A>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    collect_active_timestamps_in_objects(
        section,
        &citation.prefix,
        context,
        category.clone(),
        entries,
    );
    collect_active_timestamps_in_objects(
        section,
        &citation.suffix,
        context,
        category.clone(),
        entries,
    );
    for reference in &citation.references {
        collect_active_timestamps_in_cite_reference(
            section,
            reference,
            context,
            category.clone(),
            entries,
        );
    }
}

fn collect_active_timestamps_in_cite_reference<A: Clone>(
    section: &Section<A>,
    reference: &CiteReference<A>,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    collect_active_timestamps_in_objects(
        section,
        &reference.prefix,
        context,
        category.clone(),
        entries,
    );
    collect_active_timestamps_in_objects(section, &reference.suffix, context, category, entries);
}

fn collect_active_timestamp<A: Clone>(
    section: &Section<A>,
    ann: &A,
    timestamp: &Timestamp,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    match timestamp.kind {
        TimestampKind::Active => {}
        TimestampKind::Inactive if context.query.include_inactive_timestamps => {}
        TimestampKind::Diary if context.query.include_diary_timestamps => {
            collect_diary_timestamp(section, ann, timestamp, context, category, entries);
            return;
        }
        TimestampKind::Inactive | TimestampKind::Diary => return,
    }
    let Some(moment) = &timestamp.start else {
        return;
    };

    let base_start = AgendaDate::from_moment(moment);
    let base_end = timestamp.end.as_ref().map(AgendaDate::from_moment);
    let time = AgendaTime::from_moment(moment);
    let end_time = timestamp.end.as_ref().and_then(AgendaTime::from_moment);

    for span in occurrence_spans(
        timestamp,
        base_start,
        base_end,
        context.end,
        context.query.expand_repeaters,
    ) {
        let seed = EntrySeed {
            ann: ann.clone(),
            section,
            kind: AgendaEntryKind::Timestamp,
            timestamp,
            target_date: span.start,
            target_end_date: span.end,
            time,
            end_time,
            headline_time: headline_time(section, context.query),
            category: category.clone(),
            occurrence: span.occurrence,
        };
        collect_span_entries(&seed, context.start, context.end, entries);
    }
}

fn collect_diary_timestamp<A: Clone>(
    section: &Section<A>,
    ann: &A,
    timestamp: &Timestamp,
    context: AgendaCollectContext<'_>,
    category: Option<AgendaCategory>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    let Some((month, day)) = diary_date_month_day(timestamp.raw.as_str()) else {
        return;
    };
    for target_date in diary_dates_in_range(month, day, context.start, context.end) {
        let seed = EntrySeed {
            ann: ann.clone(),
            section,
            kind: AgendaEntryKind::Diary,
            timestamp,
            target_date,
            target_end_date: None,
            time: None,
            end_time: None,
            headline_time: headline_time(section, context.query),
            category: category.clone(),
            occurrence: AgendaOccurrence::Source,
        };
        entries.push(entry(&seed, target_date, None, None));
    }
}

fn diary_date_month_day(raw: &str) -> Option<(u8, u8)> {
    let start = raw.find("diary-date")? + "diary-date".len();
    let mut parts = raw[start..]
        .split(|ch: char| ch.is_whitespace() || matches!(ch, ')' | '>'))
        .filter(|part| !part.is_empty());
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

fn diary_dates_in_range(month: u8, day: u8, start: AgendaDate, end: AgendaDate) -> Vec<AgendaDate> {
    (start.year..=end.year)
        .map(|year| AgendaDate::new(year, month, day))
        .filter(|date| *date >= start && *date <= end)
        .collect()
}

#[derive(Clone, Copy)]
struct AgendaSpan {
    start: AgendaDate,
    end: Option<AgendaDate>,
    occurrence: AgendaOccurrence,
}

struct EntrySeed<'a, A> {
    ann: A,
    section: &'a Section<A>,
    kind: AgendaEntryKind,
    timestamp: &'a Timestamp,
    target_date: AgendaDate,
    target_end_date: Option<AgendaDate>,
    time: Option<AgendaTime>,
    end_time: Option<AgendaTime>,
    headline_time: Option<HeadlineTimeSpec>,
    category: Option<AgendaCategory>,
    occurrence: AgendaOccurrence,
}

fn collect_span_entries<A: Clone>(
    seed: &EntrySeed<'_, A>,
    start: AgendaDate,
    end: AgendaDate,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    let span_end = seed.target_end_date.unwrap_or(seed.target_date);
    let first = start.max(seed.target_date);
    let last = end.min(span_end);

    if first > last {
        return;
    }

    let mut display_date = first;
    while display_date <= last {
        entries.push(entry(seed, display_date, None, None));
        display_date = display_date.add_days(1);
    }
}

fn collect_scheduled_entries<A: Clone>(
    seed: &EntrySeed<'_, A>,
    start: AgendaDate,
    end: AgendaDate,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    let span_end = seed.target_end_date.unwrap_or(seed.target_date);
    let visible_start = scheduled_visible_start(
        seed.target_date,
        seed.timestamp.warning.as_ref(),
        seed.occurrence,
    );

    if span_end < start {
        if visible_start <= start {
            entries.push(entry(
                seed,
                start,
                Some(AgendaScheduleState::PastDue {
                    days_overdue: span_end.days_until(start) as u32,
                }),
                None,
            ));
        }
        return;
    }

    let display_end = span_end.max(visible_start);
    let first = start.max(visible_start);
    let last = end.min(display_end);

    if first > last {
        return;
    }

    let mut display_date = first;
    while display_date <= last {
        let scheduled = if visible_start > seed.target_date && display_date == visible_start {
            Some(AgendaScheduleState::Delayed {
                days_delayed: seed.target_date.days_until(visible_start) as u32,
            })
        } else if display_date > span_end {
            Some(AgendaScheduleState::PastDue {
                days_overdue: span_end.days_until(display_date) as u32,
            })
        } else {
            Some(AgendaScheduleState::OnDate)
        };
        entries.push(entry(seed, display_date, scheduled, None));
        display_date = display_date.add_days(1);
    }
}

fn collect_deadline_entries<A: Clone>(
    seed: EntrySeed<'_, A>,
    context: AgendaCollectContext<'_>,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    if context.query.include_overdue_deadlines && seed.target_date < context.start {
        entries.push(entry(
            &seed,
            context.start,
            None,
            Some(AgendaDeadlineState::Overdue {
                days_overdue: seed.target_date.days_until(context.start) as u32,
            }),
        ));
        return;
    }

    let visible_start = if context.query.include_deadline_warnings {
        warning_start(seed.target_date, seed.timestamp.warning.as_ref())
    } else {
        seed.target_date
    };
    let first = context.start.max(visible_start);
    let last = context.end.min(seed.target_date);

    if first > last {
        return;
    }

    let mut display_date = first;
    while display_date <= last {
        let deadline = if display_date < seed.target_date {
            Some(AgendaDeadlineState::Warning {
                days_until: display_date.days_until(seed.target_date) as u32,
            })
        } else {
            Some(AgendaDeadlineState::Due)
        };
        entries.push(entry(&seed, display_date, None, deadline));
        display_date = display_date.add_days(1);
    }
}

fn occurrence_spans(
    timestamp: &Timestamp,
    base_start: AgendaDate,
    base_end: Option<AgendaDate>,
    target_end: AgendaDate,
    expand_repeaters: bool,
) -> Vec<AgendaSpan> {
    let normalized_end = base_end.filter(|end| *end >= base_start);
    let mut spans = vec![AgendaSpan {
        start: base_start,
        end: normalized_end,
        occurrence: AgendaOccurrence::Source,
    }];
    let Some(repeater) = &timestamp.repeater else {
        return spans;
    };
    if !expand_repeaters || repeater.value == 0 {
        return spans;
    }

    let mut index = 1;
    let mut current_start = base_start;
    let mut current_end = normalized_end;
    while let Some(next_start) = current_start.add_interval(repeater.value as i32, repeater.unit) {
        if next_start <= current_start || next_start > target_end || index > 4_096 {
            break;
        }
        current_end =
            current_end.and_then(|end| end.add_interval(repeater.value as i32, repeater.unit));
        spans.push(AgendaSpan {
            start: next_start,
            end: current_end,
            occurrence: AgendaOccurrence::Repeater { index },
        });
        current_start = next_start;
        index += 1;
    }
    spans
}

fn entry<A: Clone>(
    seed: &EntrySeed<'_, A>,
    display_date: AgendaDate,
    scheduled: Option<AgendaScheduleState>,
    deadline: Option<AgendaDeadlineState>,
) -> AgendaEntry<A> {
    AgendaEntry {
        ann: seed.ann.clone(),
        kind: seed.kind,
        display_date,
        target_date: seed.target_date,
        target_end_date: seed.target_end_date,
        time: seed.time.or(seed.headline_time.map(|time| time.start)),
        end_time: seed
            .end_time
            .or(seed.headline_time.and_then(|time| time.end)),
        title: seed.section.title.clone(),
        raw_title: seed.section.raw_title.trim_end().to_string(),
        category: seed.category.clone(),
        level: seed.section.level,
        priority: seed.section.priority.clone(),
        todo: seed.section.todo.clone(),
        tags: seed.section.tags.clone(),
        effective_tags: seed.section.effective_tags.clone(),
        anchor: seed.section.anchor.clone(),
        timestamp: seed.timestamp.clone(),
        occurrence: seed.occurrence,
        scheduled,
        deadline,
    }
}

fn document_category<A>(document: &Document<A>) -> Option<AgendaCategory> {
    property_category(&document.properties).or_else(|| keyword_category(&document.children))
}

fn section_category<A>(section: &Section<A>) -> Option<AgendaCategory> {
    property_category(&section.properties)
}

fn property_category<A>(properties: &[Property<A>]) -> Option<AgendaCategory> {
    properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case("CATEGORY"))
        .map(|property| property.value.trim())
        .filter(|value| !value.is_empty())
        .map(AgendaCategory::new)
}

fn keyword_category<A>(elements: &[Element<A>]) -> Option<AgendaCategory> {
    elements
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Keyword(keyword) if keyword.key.eq_ignore_ascii_case("CATEGORY") => {
                Some(keyword.value.trim())
            }
            _ => None,
        })
        .find(|value| !value.is_empty())
        .map(AgendaCategory::new)
}

fn kind_order(kind: AgendaEntryKind) -> u8 {
    match kind {
        AgendaEntryKind::Deadline => 0,
        AgendaEntryKind::Scheduled => 1,
        AgendaEntryKind::Timestamp => 2,
        AgendaEntryKind::Diary => 3,
        AgendaEntryKind::Closed => 4,
    }
}
