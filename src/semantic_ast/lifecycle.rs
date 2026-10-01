//! Opt-in lifecycle projection over ordinary Org LOGBOOK and archive metadata.

use super::aot_drawer_projection::drawer_body;
use super::{
    AstRef, Document, Drawer, Element, ElementData, LifecycleRecord, LifecycleRecordKind,
    ObjectData, OrgDuration, ParsedAnnotation, Section,
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
                    collect_logbook_records(section, drawer, &element.ann, records);
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
    drawer: &Drawer<ParsedAnnotation>,
    ann: &ParsedAnnotation,
    records: &mut Vec<LifecycleRecord<ParsedAnnotation>>,
) {
    let Some(body_range) = ann.drawer_body_range else {
        return;
    };
    let mut line_start = usize::from(body_range.start());
    let mut link_sources = None;
    for source_line in drawer_body(ann).split_inclusive('\n') {
        let line = source_line.strip_suffix('\n').unwrap_or(source_line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let line_end = line_start + line.len();
        let kind =
            lifecycle_record_kind(line, line_start..line_end, drawer, ann, &mut link_sources);
        line_start += source_line.len();
        let Some(kind) = kind else {
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

fn lifecycle_record_kind(
    line: &str,
    source_range: std::ops::Range<usize>,
    drawer: &Drawer<ParsedAnnotation>,
    ann: &ParsedAnnotation,
    link_sources: &mut Option<Vec<(usize, usize)>>,
) -> Option<LifecycleRecordKind> {
    let line = trim_logbook_line(line)?;
    Some(match crate::org_aot::logbook_line_kind(line) {
        "state" => state_change_record(line),
        "refile" => LifecycleRecordKind::Refile {
            target: aot_link_source_in_line(drawer, ann, source_range, link_sources),
            timestamp: first_timestamp_raw(line),
        },
        "reschedule" => {
            let timestamps = timestamps_raw(line);
            LifecycleRecordKind::Reschedule {
                from: timestamps.first().cloned(),
                to: timestamps.get(1).cloned(),
                timestamp: timestamps.last().cloned(),
            }
        }
        "redeadline" => {
            let timestamps = timestamps_raw(line);
            LifecycleRecordKind::Redeadline {
                from: timestamps.first().cloned(),
                to: timestamps.get(1).cloned(),
                timestamp: timestamps.last().cloned(),
            }
        }
        "clock" => clock_record(line),
        _ => LifecycleRecordKind::Note {
            timestamp: first_timestamp_raw(line),
        },
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
    let Some((to, from)) = crate::org_aot::logbook_state_values(line) else {
        return LifecycleRecordKind::MalformedLogbook {
            reason: "state-change LOGBOOK line is missing quoted TODO states".to_string(),
        };
    };
    LifecycleRecordKind::StateChange {
        to: Some(to.to_string()),
        from: Some(from.to_string()),
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

fn aot_link_source_in_line(
    drawer: &Drawer<ParsedAnnotation>,
    ann: &ParsedAnnotation,
    source_range: std::ops::Range<usize>,
    link_sources: &mut Option<Vec<(usize, usize)>>,
) -> Option<String> {
    // Most LOGBOOK drawers have no refile target; visit AOT objects only on demand.
    let links = link_sources.get_or_insert_with(|| aot_link_sources(drawer));
    let first = links.partition_point(|(start, _)| *start < source_range.start);
    links[first..]
        .iter()
        .take_while(|(start, _)| *start < source_range.end)
        .find(|(_, end)| *end <= source_range.end)
        .and_then(|(start, end)| {
            let drawer_start = usize::from(ann.range.start());
            ann.raw
                .get(start.checked_sub(drawer_start)?..end.checked_sub(drawer_start)?)
                .map(str::to_owned)
        })
}

fn aot_link_sources(drawer: &Drawer<ParsedAnnotation>) -> Vec<(usize, usize)> {
    let mut links = Vec::new();
    for child in &drawer.children {
        child.visit_with(&mut |node| {
            let AstRef::Object(object) = node else {
                return;
            };
            if !matches!(object.data, ObjectData::Link(_)) {
                return;
            }
            let start = usize::from(object.ann.range.start());
            let end = usize::from(object.ann.range.end());
            links.push((start, end));
        });
    }
    links.sort_unstable_by_key(|(start, _)| *start);
    links
}
