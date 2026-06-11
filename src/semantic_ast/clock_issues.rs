//! Clock consistency diagnostics over parsed CLOCK intervals.

use super::clock_table_time::{
    clock_end_bound, clock_end_minute, clock_start_bound, clock_start_minute,
};
use super::{
    Clock, ClockIssueClock, ClockIssueFinding, ClockIssueFindingKind, ClockIssueProfile, Document,
    Element, ElementData, ParsedAnnotation, Section, SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Projects Org agenda-style clock consistency diagnostics with default Org settings.
    ///
    /// This is a read-only projection. It does not stop running clocks, change
    /// clock durations, or rewrite LOGBOOK contents.
    pub fn clock_issue_findings(&self) -> Vec<ClockIssueFinding> {
        self.clock_issue_findings_with_profile(&ClockIssueProfile::org_default())
    }

    /// Projects Org agenda-style clock consistency diagnostics with a caller profile.
    pub fn clock_issue_findings_with_profile(
        &self,
        profile: &ClockIssueProfile,
    ) -> Vec<ClockIssueFinding> {
        let mut entries = clock_issue_entries(self);
        entries.sort_by_key(|entry| entry.source.range_start);
        clock_issue_findings_from_entries(&entries, profile)
    }
}

#[derive(Clone, Debug)]
struct ClockIssueEntry {
    source: SectionIndexSource,
    outline_path: Vec<String>,
    level: usize,
    title: String,
    clock: ClockIssueClock,
    start_minute: Option<i64>,
    end_minute: Option<i64>,
    interval_seconds: Option<u64>,
    has_unparsed_duration: bool,
}

fn clock_issue_entries(document: &Document<ParsedAnnotation>) -> Vec<ClockIssueEntry> {
    let mut entries = Vec::new();
    collect_clock_issue_entries_from_elements(&document.children, &[], 0, "", &mut entries);
    for section in &document.sections {
        collect_clock_issue_entries_from_section(section, &[], &mut entries);
    }
    entries
}

fn collect_clock_issue_entries_from_section(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: &[String],
    entries: &mut Vec<ClockIssueEntry>,
) {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path.to_vec();
    outline_path.push(title.clone());

    collect_clock_issue_entries_from_elements(
        &section.children,
        &outline_path,
        section.level,
        &title,
        entries,
    );

    for child in &section.subsections {
        collect_clock_issue_entries_from_section(child, &outline_path, entries);
    }
}

fn collect_clock_issue_entries_from_elements(
    elements: &[Element<ParsedAnnotation>],
    outline_path: &[String],
    level: usize,
    title: &str,
    entries: &mut Vec<ClockIssueEntry>,
) {
    for element in elements {
        match &element.data {
            ElementData::Clock(clock) => {
                entries.push(clock_issue_entry(
                    element,
                    clock,
                    outline_path,
                    level,
                    title,
                ));
            }
            ElementData::Drawer(drawer) => collect_clock_issue_entries_from_elements(
                &drawer.children,
                outline_path,
                level,
                title,
                entries,
            ),
            ElementData::List(list) => {
                for item in &list.items {
                    collect_clock_issue_entries_from_elements(
                        &item.children,
                        outline_path,
                        level,
                        title,
                        entries,
                    );
                }
            }
            ElementData::Block(block) => collect_clock_issue_entries_from_elements(
                &block.children,
                outline_path,
                level,
                title,
                entries,
            ),
            ElementData::FootnoteDef(footnote) => collect_clock_issue_entries_from_elements(
                &footnote.children,
                outline_path,
                level,
                title,
                entries,
            ),
            ElementData::Inlinetask(task) => collect_clock_issue_entries_from_elements(
                &task.children,
                outline_path,
                level,
                title,
                entries,
            ),
            ElementData::Paragraph(_)
            | ElementData::Keyword(_)
            | ElementData::BabelCall(_)
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

fn clock_issue_entry(
    element: &Element<ParsedAnnotation>,
    clock: &Clock,
    outline_path: &[String],
    level: usize,
    title: &str,
) -> ClockIssueEntry {
    let source = SectionIndexSource::from_annotation(&element.ann);
    let start_minute = clock_start_minute(clock);
    let end_minute = clock_end_minute(clock);
    let interval_seconds = start_minute
        .zip(end_minute)
        .and_then(|(start, end)| (end > start).then_some((end - start) as u64 * 60));
    let duration_seconds = interval_seconds.or_else(|| {
        clock
            .parsed_duration
            .as_ref()
            .map(|duration| duration.total_seconds)
    });

    ClockIssueEntry {
        source: source.clone(),
        outline_path: outline_path.to_vec(),
        level,
        title: title.to_string(),
        clock: ClockIssueClock {
            source,
            raw: clock.raw.clone(),
            start: clock_start_bound(clock),
            end: clock_end_bound(clock),
            duration_seconds,
        },
        start_minute,
        end_minute,
        interval_seconds,
        has_unparsed_duration: clock.duration.is_some() && clock.parsed_duration.is_none(),
    }
}

fn clock_issue_findings_from_entries(
    entries: &[ClockIssueEntry],
    profile: &ClockIssueProfile,
) -> Vec<ClockIssueFinding> {
    entries
        .iter()
        .scan(None, |previous_closed, entry| {
            let finding = clock_issue_finding(entry, previous_closed.as_ref(), profile);
            if entry.start_minute.is_some() && entry.end_minute.is_some() {
                *previous_closed = Some(entry.clone());
            }
            Some(finding)
        })
        .flatten()
        .collect()
}

fn clock_issue_finding(
    entry: &ClockIssueEntry,
    previous_closed: Option<&ClockIssueEntry>,
    profile: &ClockIssueProfile,
) -> Option<ClockIssueFinding> {
    let Some(start_minute) = entry.start_minute else {
        return Some(finding(
            entry,
            ClockIssueFindingKind::InvalidClock,
            "No valid clock start timestamp".to_string(),
            None,
            None,
            None,
        ));
    };

    let Some(_end_minute) = entry.end_minute else {
        return Some(finding(
            entry,
            ClockIssueFindingKind::NoEndTime,
            "No end time".to_string(),
            None,
            None,
            None,
        ));
    };

    if entry.has_unparsed_duration {
        return Some(finding(
            entry,
            ClockIssueFindingKind::InvalidDuration,
            "Clock duration token is present but could not be parsed".to_string(),
            None,
            None,
            None,
        ));
    }

    let Some(duration_seconds) = entry.interval_seconds else {
        return Some(finding(
            entry,
            ClockIssueFindingKind::InvalidRange,
            "Clock end is not after clock start".to_string(),
            None,
            Some(0),
            None,
        ));
    };

    if let Some(max_duration_seconds) = profile.max_duration.map(|duration| duration.as_seconds())
        && duration_seconds > max_duration_seconds
    {
        return Some(finding(
            entry,
            ClockIssueFindingKind::LongDuration,
            format!(
                "Clocking interval is very long: {}",
                format_duration(duration_seconds)
            ),
            None,
            Some(duration_seconds),
            Some(max_duration_seconds),
        ));
    }

    if let Some(min_duration_seconds) = profile.min_duration.map(|duration| duration.as_seconds())
        && duration_seconds < min_duration_seconds
    {
        return Some(finding(
            entry,
            ClockIssueFindingKind::ShortDuration,
            format!(
                "Clocking interval is very short: {}",
                format_duration(duration_seconds)
            ),
            None,
            Some(duration_seconds),
            Some(min_duration_seconds),
        ));
    }

    if let Some(previous) = previous_closed
        && let Some(previous_end_minute) = previous.end_minute
    {
        if start_minute < previous_end_minute {
            let overlap_seconds = (previous_end_minute - start_minute) as u64 * 60;
            return Some(finding(
                entry,
                ClockIssueFindingKind::Overlap,
                format!("Clocking overlap: {} minutes", overlap_seconds / 60),
                Some(previous.clock.clone()),
                Some(overlap_seconds),
                None,
            ));
        }

        if let Some(max_gap_seconds) = profile.max_gap.map(|duration| duration.as_seconds()) {
            let gap_seconds = (start_minute - previous_end_minute) as u64 * 60;
            if gap_seconds > max_gap_seconds
                && !gap_contains_ok_minute(
                    previous_end_minute,
                    start_minute,
                    &profile.gap_ok_around_minutes,
                )
            {
                return Some(finding(
                    entry,
                    ClockIssueFindingKind::Gap,
                    format!("Clocking gap: {} minutes", gap_seconds / 60),
                    Some(previous.clock.clone()),
                    Some(gap_seconds),
                    Some(max_gap_seconds),
                ));
            }
        }
    }

    None
}

fn finding(
    entry: &ClockIssueEntry,
    kind: ClockIssueFindingKind,
    message: String,
    previous_clock: Option<ClockIssueClock>,
    duration_seconds: Option<u64>,
    threshold_seconds: Option<u64>,
) -> ClockIssueFinding {
    ClockIssueFinding {
        source: entry.source.clone(),
        outline_path: entry.outline_path.clone(),
        level: entry.level,
        title: entry.title.clone(),
        kind,
        message,
        clock: entry.clock.clone(),
        previous_clock,
        duration_seconds,
        threshold_seconds,
    }
}

fn gap_contains_ok_minute(start_minute: i64, end_minute: i64, ok_minutes: &[u16]) -> bool {
    if ok_minutes.is_empty() || end_minute <= start_minute {
        return false;
    }

    if end_minute - start_minute >= 24 * 60 {
        return true;
    }

    let min1 = start_minute.rem_euclid(24 * 60);
    let mut min2 = end_minute.rem_euclid(24 * 60);
    if min2 < min1 {
        min2 += 24 * 60;
    }

    ok_minutes.iter().any(|minute| {
        let mut ok = i64::from(*minute);
        if ok < min1 {
            ok += 24 * 60;
        }
        min1 <= ok && ok <= min2
    })
}

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    format!("{}:{:02}", minutes / 60, minutes % 60)
}
