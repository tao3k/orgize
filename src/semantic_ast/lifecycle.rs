//! Opt-in lifecycle projection over ordinary Org LOGBOOK and archive metadata.

use super::aot_drawer_projection::drawer_body;
use super::{
    Document, Element, ElementData, LifecycleRecord, LifecycleRecordKind, OrgDuration,
    ParsedAnnotation, Section,
};

impl Document<ParsedAnnotation> {
    /// Projects Scheme-bounded LOGBOOK bodies into lifecycle records without mutating the AST.
    pub fn lifecycle_records(&self) -> Vec<LifecycleRecord<ParsedAnnotation>> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_section_lifecycle_records(section, &mut records);
        }
        records
    }
}

pub(super) fn collect_section_lifecycle_records(
    section: &Section<ParsedAnnotation>,
    records: &mut Vec<LifecycleRecord<ParsedAnnotation>>,
) {
    collect_lifecycle_records_in_elements(section, &section.children, records);
    for subsection in &section.subsections {
        collect_section_lifecycle_records(subsection, records);
    }
}

pub(super) fn section_lifecycle_records(
    section: &Section<ParsedAnnotation>,
) -> Vec<LifecycleRecord<ParsedAnnotation>> {
    let mut records = Vec::new();
    collect_lifecycle_records_in_elements(section, &section.children, &mut records);
    records
}

fn collect_lifecycle_records_in_elements(
    section: &Section<ParsedAnnotation>,
    elements: &[Element<ParsedAnnotation>],
    records: &mut Vec<LifecycleRecord<ParsedAnnotation>>,
) {
    for element in elements {
        match &element.data {
            ElementData::Drawer(drawer) => {
                if drawer.name.eq_ignore_ascii_case("LOGBOOK") {
                    collect_logbook_records(section, &element.ann, records);
                }
                collect_lifecycle_records_in_elements(section, &drawer.children, records);
            }
            ElementData::List(list) => {
                for item in &list.items {
                    collect_lifecycle_records_in_elements(section, &item.children, records);
                }
            }
            ElementData::Block(block) => {
                collect_lifecycle_records_in_elements(section, &block.children, records);
            }
            ElementData::FootnoteDef(footnote) => {
                collect_lifecycle_records_in_elements(section, &footnote.children, records);
            }
            ElementData::Inlinetask(task) => {
                collect_lifecycle_records_in_elements(section, &task.children, records);
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

fn collect_logbook_records(
    section: &Section<ParsedAnnotation>,
    ann: &ParsedAnnotation,
    records: &mut Vec<LifecycleRecord<ParsedAnnotation>>,
) {
    for line in drawer_body(ann).lines() {
        let Some(kind) = lifecycle_record_kind(line) else {
            continue;
        };
        records.push(LifecycleRecord {
            ann: ann.clone(),
            section_anchor: section.anchor.clone(),
            section_title: section.raw_title.trim_end().to_string(),
            kind,
            raw: line.trim().to_string(),
        });
    }
}

fn lifecycle_record_kind(line: &str) -> Option<LifecycleRecordKind> {
    let line = trim_logbook_line(line)?;
    if line.starts_with("State ") {
        return Some(state_change_record(line));
    }
    if line.starts_with("Note taken on") {
        return Some(LifecycleRecordKind::Note {
            timestamp: first_timestamp_raw(line),
        });
    }
    if line.starts_with("Refiled") || line.starts_with("Refiling") {
        return Some(LifecycleRecordKind::Refile {
            target: link_target_raw(line),
            timestamp: first_timestamp_raw(line),
        });
    }
    if line.starts_with("Rescheduled") {
        let timestamps = timestamps_raw(line);
        return Some(LifecycleRecordKind::Reschedule {
            from: timestamps.first().cloned(),
            to: timestamps.get(1).cloned(),
            timestamp: timestamps.last().cloned(),
        });
    }
    if line.starts_with("New deadline")
        || line.starts_with("Deadline")
        || line.starts_with("Removed deadline")
    {
        let timestamps = timestamps_raw(line);
        return Some(LifecycleRecordKind::Redeadline {
            from: timestamps.first().cloned(),
            to: timestamps.get(1).cloned(),
            timestamp: timestamps.last().cloned(),
        });
    }
    if line.starts_with("CLOCK:") {
        return Some(clock_record(line));
    }
    Some(LifecycleRecordKind::Note {
        timestamp: first_timestamp_raw(line),
    })
}

fn trim_logbook_line(line: &str) -> Option<&str> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    Some(line.strip_prefix('-').map(str::trim_start).unwrap_or(line))
}

fn state_change_record(line: &str) -> LifecycleRecordKind {
    let quoted = quoted_segments(line);
    if quoted.len() < 2 {
        return LifecycleRecordKind::MalformedLogbook {
            reason: "state-change LOGBOOK line is missing quoted TODO states".to_string(),
        };
    }
    LifecycleRecordKind::StateChange {
        to: quoted.first().cloned(),
        from: quoted.get(1).cloned(),
        timestamp: first_timestamp_raw(line),
    }
}

fn clock_record(line: &str) -> LifecycleRecordKind {
    let duration = line
        .split_once("=>")
        .and_then(|(_, duration)| OrgDuration::parse(duration.trim().to_string()));
    if line.contains("=>") && duration.is_none() {
        return LifecycleRecordKind::MalformedLogbook {
            reason: "CLOCK LOGBOOK line has an invalid duration summary".to_string(),
        };
    }
    LifecycleRecordKind::Clock {
        duration,
        timestamp: first_timestamp_raw(line),
    }
}

fn quoted_segments(line: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('"') else {
            break;
        };
        segments.push(rest[..end].to_string());
        rest = &rest[end + 1..];
    }
    segments
}

fn timestamps_raw(line: &str) -> Vec<String> {
    let mut timestamps = Vec::new();
    let mut rest = line;
    while let Some((timestamp, next)) = timestamp_raw(rest) {
        timestamps.push(timestamp.to_string());
        rest = &rest[next..];
    }
    timestamps
}

fn first_timestamp_raw(line: &str) -> Option<String> {
    timestamp_raw(line).map(|(timestamp, _)| timestamp.to_string())
}

fn timestamp_raw(line: &str) -> Option<(&str, usize)> {
    let start = line.find(['<', '['])?;
    let close = match line.as_bytes()[start] {
        b'<' => '>',
        b'[' => ']',
        _ => return None,
    };
    let end = line[start..].find(close)? + start + 1;
    Some((&line[start..end], end))
}

fn link_target_raw(line: &str) -> Option<String> {
    let start = line.find("[[")?;
    let end = line[start + 2..].find("]]")? + start + 4;
    Some(line[start..end].to_string())
}
