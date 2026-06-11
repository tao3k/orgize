//! Habit metadata projection over ordinary Org agenda constructs.

use super::lifecycle::section_lifecycle_records;
use super::{
    Document, Element, ElementData, HabitConsistency, HabitLastRepeat, HabitRecord,
    LifecycleRecordKind, OrgDuration, ParsedAnnotation, Section, SectionIndexSource, Timestamp,
    TimestampRepeater,
};

impl Document<ParsedAnnotation> {
    /// Projects `STYLE: habit` headlines into reusable agenda metadata.
    ///
    /// This intentionally exposes graph inputs only. It does not render the
    /// Emacs agenda habit graph or execute agenda commands.
    pub fn habit_records(&self) -> Vec<HabitRecord> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_section_habits(section, &mut records);
        }
        records
    }
}

fn collect_section_habits(section: &Section<ParsedAnnotation>, records: &mut Vec<HabitRecord>) {
    if is_habit_section(section) {
        records.push(habit_record(section));
    }
    for subsection in &section.subsections {
        collect_section_habits(subsection, records);
    }
}

fn habit_record(section: &Section<ParsedAnnotation>) -> HabitRecord {
    let scheduled = section.planning.scheduled.clone();
    let deadline = section.planning.deadline.clone();
    let repeater = habit_repeater(scheduled.as_ref(), deadline.as_ref());
    let last_repeat = habit_last_repeat(section);
    let effort = habit_effort(section);
    let (clock_count, clock_total_seconds) = habit_clock_summary(section);
    let consistency =
        habit_consistency(scheduled.as_ref(), repeater.as_ref(), last_repeat.as_ref());

    HabitRecord {
        source: SectionIndexSource::from_annotation(&section.ann),
        level: section.level,
        title: section.raw_title.trim_end().to_string(),
        todo: section.todo.clone(),
        tags: section.tags.clone(),
        effective_tags: section.effective_tags.clone(),
        scheduled,
        deadline,
        repeater,
        last_repeat,
        effort,
        clock_count,
        clock_total_seconds,
        consistency,
    }
}

fn is_habit_section(section: &Section<ParsedAnnotation>) -> bool {
    section.effective_properties.iter().any(|property| {
        property.key.eq_ignore_ascii_case("STYLE")
            && property.value.trim().eq_ignore_ascii_case("habit")
    })
}

fn habit_repeater(
    scheduled: Option<&Timestamp>,
    deadline: Option<&Timestamp>,
) -> Option<TimestampRepeater> {
    scheduled
        .and_then(|timestamp| timestamp.repeater.clone())
        .or_else(|| deadline.and_then(|timestamp| timestamp.repeater.clone()))
}

fn habit_last_repeat(section: &Section<ParsedAnnotation>) -> Option<HabitLastRepeat> {
    section
        .properties
        .iter()
        .chain(section.effective_properties.iter())
        .find(|property| {
            property.key.eq_ignore_ascii_case("LAST_REPEAT") && !property.value.trim().is_empty()
        })
        .map(|property| HabitLastRepeat {
            source: SectionIndexSource::from_annotation(&property.ann),
            raw: property.value.clone(),
        })
}

fn habit_effort(section: &Section<ParsedAnnotation>) -> Option<OrgDuration> {
    section
        .effective_properties
        .iter()
        .find(|property| property.is_effort())
        .and_then(|property| property.duration.clone())
}

fn habit_clock_summary(section: &Section<ParsedAnnotation>) -> (usize, u64) {
    lifecycle_clock_durations(section)
        .into_iter()
        .chain(direct_clock_durations(&section.children, false))
        .fold((0, 0), |(count, total), seconds| {
            (count + 1, total + seconds)
        })
}

fn lifecycle_clock_durations(section: &Section<ParsedAnnotation>) -> Vec<u64> {
    section_lifecycle_records(section)
        .into_iter()
        .filter_map(|record| match record.kind {
            LifecycleRecordKind::Clock {
                duration: Some(duration),
                ..
            } => Some(duration.total_seconds),
            _ => None,
        })
        .collect()
}

fn direct_clock_durations(elements: &[Element<ParsedAnnotation>], in_logbook: bool) -> Vec<u64> {
    let mut durations = Vec::new();
    collect_direct_clock_durations(elements, in_logbook, &mut durations);
    durations
}

fn collect_direct_clock_durations(
    elements: &[Element<ParsedAnnotation>],
    in_logbook: bool,
    durations: &mut Vec<u64>,
) {
    for element in elements {
        match &element.data {
            ElementData::Clock(clock) if !in_logbook => {
                if let Some(duration) = clock.parsed_duration.as_ref() {
                    durations.push(duration.total_seconds);
                }
            }
            ElementData::Drawer(drawer) => {
                collect_direct_clock_durations(
                    &drawer.children,
                    in_logbook || drawer.name.eq_ignore_ascii_case("LOGBOOK"),
                    durations,
                );
            }
            ElementData::List(list) => {
                for item in &list.items {
                    collect_direct_clock_durations(&item.children, in_logbook, durations);
                }
            }
            ElementData::Block(block) => {
                collect_direct_clock_durations(&block.children, in_logbook, durations);
            }
            ElementData::FootnoteDef(footnote) => {
                collect_direct_clock_durations(&footnote.children, in_logbook, durations);
            }
            ElementData::Inlinetask(task) => {
                collect_direct_clock_durations(&task.children, in_logbook, durations);
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
}

fn habit_consistency(
    scheduled: Option<&Timestamp>,
    repeater: Option<&TimestampRepeater>,
    last_repeat: Option<&HabitLastRepeat>,
) -> HabitConsistency {
    if scheduled.is_none() {
        HabitConsistency::MissingScheduled
    } else if repeater.is_none() {
        HabitConsistency::MissingRepeater
    } else if last_repeat.is_none() {
        HabitConsistency::MissingLastRepeat
    } else {
        HabitConsistency::Complete
    }
}
