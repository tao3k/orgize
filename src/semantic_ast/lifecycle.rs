//! Opt-in lifecycle projection over ordinary Org LOGBOOK and archive metadata.

use rowan::TextRange;

use super::aot_drawer_projection::drawer_body;
use super::{
    AstRef, Document, Drawer, Element, ElementData, LifecycleRecord, LifecycleRecordKind,
    ObjectData, OrgDuration, ParsedAnnotation, Section,
};

#[derive(Clone, Copy)]
struct SourceSpan {
    start: usize,
    end: usize,
}

impl From<TextRange> for SourceSpan {
    fn from(range: TextRange) -> Self {
        Self {
            start: usize::from(range.start()),
            end: usize::from(range.end()),
        }
    }
}

struct ClockFact {
    start: usize,
    first_timestamp: Option<SourceSpan>,
    has_duration: bool,
    duration: Option<OrgDuration>,
}

#[derive(Default)]
struct LogbookFacts {
    links: Vec<SourceSpan>,
    timestamps: Vec<SourceSpan>,
    clocks: Vec<ClockFact>,
}

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
    let facts = aot_logbook_facts(drawer);
    for source_line in drawer_body(ann).split_inclusive('\n') {
        let line = source_line.strip_suffix('\n').unwrap_or(source_line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let line_end = line_start + line.len();
        let kind = lifecycle_record_kind(line, line_start..line_end, &facts, ann);
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
    facts: &LogbookFacts,
    ann: &ParsedAnnotation,
) -> Option<LifecycleRecordKind> {
    let line = trim_logbook_line(line)?;
    let timestamps = facts.timestamp_sources_in_line(ann, &source_range);
    Some(match crate::org_aot::logbook_line_kind(line) {
        "state" => state_change_record(line, timestamps.first().cloned()),
        "refile" => LifecycleRecordKind::Refile {
            target: facts.first_link_source_in_line(ann, &source_range),
            timestamp: timestamps.first().cloned(),
        },
        "reschedule" => LifecycleRecordKind::Reschedule {
            from: timestamps.first().cloned(),
            to: timestamps.get(1).cloned(),
            timestamp: timestamps.last().cloned(),
        },
        "redeadline" => LifecycleRecordKind::Redeadline {
            from: timestamps.first().cloned(),
            to: timestamps.get(1).cloned(),
            timestamp: timestamps.last().cloned(),
        },
        "clock" => clock_record(
            line,
            facts.clock_in_line(&source_range),
            ann,
            timestamps.first().cloned(),
        ),
        _ => LifecycleRecordKind::Note {
            timestamp: timestamps.first().cloned(),
        },
    })
}

fn trim_logbook_line(line: &str) -> Option<&str> {
    if line.trim().is_empty() {
        return None;
    }
    Some(crate::org_aot::logbook_content_line(line))
}

fn state_change_record(line: &str, timestamp: Option<String>) -> LifecycleRecordKind {
    let Some((to, from)) = crate::org_aot::logbook_state_values(line) else {
        return LifecycleRecordKind::MalformedLogbook {
            reason: "state-change LOGBOOK line is missing quoted TODO states".to_string(),
        };
    };
    LifecycleRecordKind::StateChange {
        to: Some(to.to_string()),
        from: Some(from.to_string()),
        timestamp,
    }
}

fn clock_record(
    line: &str,
    clock: Option<&ClockFact>,
    ann: &ParsedAnnotation,
    fallback_timestamp: Option<String>,
) -> LifecycleRecordKind {
    let (has_duration, duration) = if let Some(clock) = clock {
        (clock.has_duration, clock.duration.clone())
    } else {
        let value = crate::org_aot::logbook_clock_duration_value(line);
        (value.is_some(), value.and_then(OrgDuration::parse))
    };
    if has_duration && duration.is_none() {
        return LifecycleRecordKind::MalformedLogbook {
            reason: "CLOCK LOGBOOK line has an invalid duration summary".to_string(),
        };
    }
    LifecycleRecordKind::Clock {
        duration,
        timestamp: clock
            .and_then(|clock| clock.first_timestamp)
            .and_then(|span| source_span_raw(ann, span))
            .or(fallback_timestamp),
    }
}

fn source_span_raw(ann: &ParsedAnnotation, span: SourceSpan) -> Option<String> {
    let drawer_start = usize::from(ann.range.start());
    ann.raw
        .get(span.start.checked_sub(drawer_start)?..span.end.checked_sub(drawer_start)?)
        .map(str::to_owned)
}

impl LogbookFacts {
    fn timestamp_sources_in_line(
        &self,
        ann: &ParsedAnnotation,
        source_range: &std::ops::Range<usize>,
    ) -> Vec<String> {
        let first = self
            .timestamps
            .partition_point(|span| span.start < source_range.start);
        self.timestamps[first..]
            .iter()
            .take_while(|span| span.start < source_range.end)
            .filter(|span| span.end <= source_range.end)
            .filter_map(|span| source_span_raw(ann, *span))
            .collect()
    }

    fn first_link_source_in_line(
        &self,
        ann: &ParsedAnnotation,
        source_range: &std::ops::Range<usize>,
    ) -> Option<String> {
        let first = self
            .links
            .partition_point(|span| span.start < source_range.start);
        self.links[first..]
            .iter()
            .take_while(|span| span.start < source_range.end)
            .find(|span| span.end <= source_range.end)
            .and_then(|span| source_span_raw(ann, *span))
    }

    fn clock_in_line(&self, source_range: &std::ops::Range<usize>) -> Option<&ClockFact> {
        let first = self
            .clocks
            .partition_point(|clock| clock.start < source_range.start);
        self.clocks
            .get(first)
            .filter(|clock| clock.start < source_range.end)
    }
}

fn aot_logbook_facts(drawer: &Drawer<ParsedAnnotation>) -> LogbookFacts {
    let mut facts = LogbookFacts::default();
    for child in &drawer.children {
        child.visit_with(&mut |node| match node {
            AstRef::Object(object) => match &object.data {
                ObjectData::Link(_) => facts.links.push(object.ann.range.into()),
                ObjectData::Timestamp(_) => {
                    facts.timestamps.push(
                        object
                            .ann
                            .timestamp_first_point_range
                            .unwrap_or(object.ann.range)
                            .into(),
                    );
                    if let Some(second) = object.ann.timestamp_second_point_range {
                        facts.timestamps.push(second.into());
                    }
                }
                _ => {}
            },
            AstRef::Element(element) => {
                if let ElementData::Clock(clock) = &element.data {
                    facts.clocks.push(ClockFact {
                        start: usize::from(element.ann.range.start()),
                        first_timestamp: element.ann.timestamp_first_point_range.map(Into::into),
                        has_duration: clock.duration.is_some(),
                        duration: clock.parsed_duration.clone(),
                    });
                }
            }
            _ => {}
        });
    }
    facts.links.sort_unstable_by_key(|span| span.start);
    facts.timestamps.sort_unstable_by_key(|span| span.start);
    facts.clocks.sort_unstable_by_key(|clock| clock.start);
    facts
}
