//! Semantic agenda projection from headline planning timestamps.

use super::agenda_model::{
    is_done_keyword, warning_start, AgendaDate, AgendaDeadlineState, AgendaEntry, AgendaEntryKind,
    AgendaOccurrence, AgendaQuery, AgendaTime,
};
use super::model::{Document, Section, Timestamp};

impl<A: Clone> Document<A> {
    /// Projects headline planning timestamps into agenda rows.
    ///
    /// This is an opt-in semantic view: it does not mutate the parsed AST and it
    /// does not change the lossless syntax/export substrate.
    pub fn agenda_entries(&self, query: &AgendaQuery) -> Vec<AgendaEntry<A>> {
        let mut entries = Vec::new();
        let (start, end) = query.bounds();

        for section in &self.sections {
            collect_section(section, query, start, end, &mut entries);
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

fn collect_section<A: Clone>(
    section: &Section<A>,
    query: &AgendaQuery,
    start: AgendaDate,
    end: AgendaDate,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    if section_matches_query(section, query) {
        if query.include_scheduled {
            collect_timestamp(
                section,
                AgendaEntryKind::Scheduled,
                section.planning.scheduled.as_ref(),
                query,
                start,
                end,
                entries,
            );
        }

        if query.include_deadlines {
            collect_timestamp(
                section,
                AgendaEntryKind::Deadline,
                section.planning.deadline.as_ref(),
                query,
                start,
                end,
                entries,
            );
        }

        if query.include_closed {
            collect_timestamp(
                section,
                AgendaEntryKind::Closed,
                section.planning.closed.as_ref(),
                query,
                start,
                end,
                entries,
            );
        }
    }

    for subsection in &section.subsections {
        collect_section(subsection, query, start, end, entries);
    }
}

fn section_matches_query<A>(section: &Section<A>, query: &AgendaQuery) -> bool {
    if section.is_comment && !query.include_comments {
        return false;
    }
    if !query.include_done && is_done_keyword(&section.todo) {
        return false;
    }
    if !query.include_archived && has_tag(&section.effective_tags, "ARCHIVE") {
        return false;
    }
    if query
        .required_tags
        .iter()
        .any(|required| !has_tag(&section.effective_tags, required))
    {
        return false;
    }
    if query
        .excluded_tags
        .iter()
        .any(|excluded| has_tag(&section.effective_tags, excluded))
    {
        return false;
    }
    true
}

fn collect_timestamp<A: Clone>(
    section: &Section<A>,
    kind: AgendaEntryKind,
    timestamp: Option<&Timestamp>,
    query: &AgendaQuery,
    start: AgendaDate,
    end: AgendaDate,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    let Some(timestamp) = timestamp else {
        return;
    };
    let Some(moment) = &timestamp.start else {
        return;
    };

    let base_date = AgendaDate::from_moment(moment);
    let time = AgendaTime::from_moment(moment);
    let target_end = match kind {
        AgendaEntryKind::Deadline if query.include_deadline_warnings => timestamp
            .warning
            .as_ref()
            .and_then(|warning| end.add_interval(warning.value as i32, warning.unit))
            .unwrap_or(end),
        _ => end,
    };
    let occurrences = occurrence_dates(timestamp, base_date, target_end, query.expand_repeaters);

    for (target_date, occurrence) in occurrences {
        let seed = EntrySeed {
            section,
            kind,
            timestamp,
            target_date,
            time,
            occurrence,
        };
        match kind {
            AgendaEntryKind::Deadline => collect_deadline_entries(seed, query, start, end, entries),
            AgendaEntryKind::Scheduled | AgendaEntryKind::Closed => {
                if (start..=end).contains(&target_date) {
                    entries.push(entry(&seed, target_date, None));
                }
            }
        }
    }
}

struct EntrySeed<'a, A> {
    section: &'a Section<A>,
    kind: AgendaEntryKind,
    timestamp: &'a Timestamp,
    target_date: AgendaDate,
    time: Option<AgendaTime>,
    occurrence: AgendaOccurrence,
}

fn collect_deadline_entries<A: Clone>(
    seed: EntrySeed<'_, A>,
    query: &AgendaQuery,
    start: AgendaDate,
    end: AgendaDate,
    entries: &mut Vec<AgendaEntry<A>>,
) {
    if query.include_overdue_deadlines && seed.target_date < start {
        entries.push(entry(
            &seed,
            start,
            Some(AgendaDeadlineState::Overdue {
                days_overdue: seed.target_date.days_until(start) as u32,
            }),
        ));
        return;
    }

    let visible_start = if query.include_deadline_warnings {
        warning_start(seed.target_date, seed.timestamp.warning.as_ref())
    } else {
        seed.target_date
    };
    let first = start.max(visible_start);
    let last = end.min(seed.target_date);

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
        entries.push(entry(&seed, display_date, deadline));
        display_date = display_date.add_days(1);
    }
}

fn occurrence_dates(
    timestamp: &Timestamp,
    base_date: AgendaDate,
    target_end: AgendaDate,
    expand_repeaters: bool,
) -> Vec<(AgendaDate, AgendaOccurrence)> {
    let mut dates = vec![(base_date, AgendaOccurrence::Source)];
    let Some(repeater) = &timestamp.repeater else {
        return dates;
    };
    if !expand_repeaters || repeater.value == 0 {
        return dates;
    }

    let mut index = 1;
    let mut current = base_date;
    while let Some(next) = current.add_interval(repeater.value as i32, repeater.unit) {
        if next <= current || next > target_end || index > 4_096 {
            break;
        }
        dates.push((next, AgendaOccurrence::Repeater { index }));
        current = next;
        index += 1;
    }
    dates
}

fn entry<A: Clone>(
    seed: &EntrySeed<'_, A>,
    display_date: AgendaDate,
    deadline: Option<AgendaDeadlineState>,
) -> AgendaEntry<A> {
    AgendaEntry {
        ann: seed.section.ann.clone(),
        kind: seed.kind,
        display_date,
        target_date: seed.target_date,
        time: seed.time,
        title: seed.section.title.clone(),
        raw_title: seed.section.raw_title.trim_end().to_string(),
        level: seed.section.level,
        todo: seed.section.todo.clone(),
        tags: seed.section.tags.clone(),
        effective_tags: seed.section.effective_tags.clone(),
        anchor: seed.section.anchor.clone(),
        timestamp: seed.timestamp.clone(),
        occurrence: seed.occurrence,
        deadline,
    }
}

fn has_tag(tags: &[String], needle: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(needle))
}

fn kind_order(kind: AgendaEntryKind) -> u8 {
    match kind {
        AgendaEntryKind::Deadline => 0,
        AgendaEntryKind::Scheduled => 1,
        AgendaEntryKind::Closed => 2,
    }
}
