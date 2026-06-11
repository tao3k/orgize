//! Non-executing projections for runtime-adjacent Org metadata.

use super::{
    Document, Element, ElementData, FeedStatusDrawerName, FeedStatusRecord, MobileFlaggedSection,
    MobileIndexLink, MobileOriginalId, MobilePriorityDeclaration, MobileProperty,
    MobileReadonlyKeyword, Object, ObjectData, ParsedAnnotation, Property, RuntimeMetadataBoundary,
    RuntimeMetadataBoundaryKind, RuntimeMetadataPlan, RuntimeMetadataWarning,
    RuntimeMetadataWarningKind, Section, SectionIndexSource, SourcePosition, TimerContext,
    TimerRecord,
};

const FEEDSTATUS_DRAWER: &str = "FEEDSTATUS";
const FLAGGED_TAG: &str = "FLAGGED";
const ORIGINAL_ID_PROPERTY: &str = "ORIGINAL_ID";

impl Document<ParsedAnnotation> {
    /// Collects source-backed metadata used by Org Feed, timers, MobileOrg,
    /// and persistence-adjacent workflows without performing I/O or mutation.
    pub fn runtime_metadata_plan(&self) -> RuntimeMetadataPlan {
        let mut plan = RuntimeMetadataPlan {
            boundaries: runtime_boundaries(),
            ..RuntimeMetadataPlan::default()
        };
        collect_mobile_keywords(self, &mut plan);
        let mobile_index_marker =
            !plan.mobile.readonly.is_empty() || !plan.mobile.all_priorities.is_empty();
        collect_elements(&self.children, &[], None, &mut plan);
        for section in &self.sections {
            collect_section(section, Vec::new(), mobile_index_marker, &mut plan);
        }
        if mobile_index_marker && plan.mobile.index_links.is_empty() {
            plan.warnings.push(RuntimeMetadataWarning {
                kind: RuntimeMetadataWarningKind::MobileReadonlyWithoutIndexLinks,
                message: "MobileOrg index-style metadata was found without any file links"
                    .to_string(),
            });
        }
        plan
    }
}

fn collect_mobile_keywords(document: &Document<ParsedAnnotation>, plan: &mut RuntimeMetadataPlan) {
    for keyword in &document.metadata {
        if keyword.key.eq_ignore_ascii_case("READONLY") {
            plan.mobile.readonly.push(MobileReadonlyKeyword {
                source: SectionIndexSource::from_annotation(&keyword.ann),
                value: keyword.value.clone(),
            });
        } else if keyword.key.eq_ignore_ascii_case("ALLPRIORITIES") {
            plan.mobile.all_priorities.push(MobilePriorityDeclaration {
                source: SectionIndexSource::from_annotation(&keyword.ann),
                values: split_words(keyword.value.as_str()),
                raw: keyword.value.clone(),
            });
        }
    }
    if plan.mobile.readonly.is_empty() || plan.mobile.all_priorities.is_empty() {
        collect_mobile_marker_lines(document, plan);
    }
}

fn collect_mobile_marker_lines(
    document: &Document<ParsedAnnotation>,
    plan: &mut RuntimeMetadataPlan,
) {
    let mut offset = 0usize;
    for (line_index, raw_line) in document.ann.raw.split_inclusive('\n').enumerate() {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        let source = source_for_line(line_index, leading, trimmed.len(), offset);
        if plan.mobile.readonly.is_empty() && trimmed.eq_ignore_ascii_case("#+READONLY") {
            plan.mobile.readonly.push(MobileReadonlyKeyword {
                source,
                value: String::new(),
            });
        } else if plan.mobile.all_priorities.is_empty()
            && trimmed
                .get(..15)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+ALLPRIORITIES"))
            && let Some((_, value)) = trimmed.split_once(':')
        {
            plan.mobile.all_priorities.push(MobilePriorityDeclaration {
                source,
                values: split_words(value),
                raw: value.trim().to_string(),
            });
        }
        offset += raw_line.len();
    }
}

fn source_for_line(
    line_index: usize,
    leading: usize,
    trimmed_len: usize,
    offset: usize,
) -> SectionIndexSource {
    let range_start = offset + leading;
    let range_end = range_start + trimmed_len;
    SectionIndexSource {
        start: SourcePosition {
            line: line_index + 1,
            column: leading + 1,
        },
        end: SourcePosition {
            line: line_index + 1,
            column: leading + trimmed_len + 1,
        },
        range_start: range_start as u32,
        range_end: range_end as u32,
    }
}

fn collect_section(
    section: &Section<ParsedAnnotation>,
    mut outline_path: Vec<String>,
    mobile_index_marker: bool,
    plan: &mut RuntimeMetadataPlan,
) {
    let title = section.raw_title.trim_end().to_string();
    outline_path.push(title.clone());
    collect_timers(
        &section.raw_title,
        SectionIndexSource::from_annotation(&section.ann),
        &outline_path,
        TimerContext::Headline,
        plan,
    );
    collect_mobile_section(section, &outline_path, mobile_index_marker, plan);
    collect_elements(&section.children, &outline_path, Some(section), plan);
    for child in &section.subsections {
        collect_section(child, outline_path.clone(), mobile_index_marker, plan);
    }
}

fn collect_mobile_section(
    section: &Section<ParsedAnnotation>,
    outline_path: &[String],
    mobile_index_marker: bool,
    plan: &mut RuntimeMetadataPlan,
) {
    let title = section.raw_title.trim_end().to_string();
    let original_id = original_id(section);
    if has_tag(&section.tags, FLAGGED_TAG) || has_tag(&section.effective_tags, FLAGGED_TAG) {
        plan.mobile.flagged_sections.push(MobileFlaggedSection {
            source: SectionIndexSource::from_annotation(&section.ann),
            outline_path: outline_path.to_vec(),
            title: title.clone(),
            original_id: original_id.as_ref().map(|(_, value)| value.clone()),
            mobile_properties: mobile_properties(&section.properties),
        });
    }
    if let Some((source, value)) = original_id {
        plan.mobile.original_ids.push(MobileOriginalId {
            source,
            outline_path: outline_path.to_vec(),
            title: title.clone(),
            value,
        });
    }
    if mobile_index_marker
        && section.level == 1
        && let Some(link) = title_file_link(&section.title, title.as_str())
    {
        plan.mobile.index_links.push(link);
    }
}

fn collect_elements(
    elements: &[Element<ParsedAnnotation>],
    outline_path: &[String],
    section: Option<&Section<ParsedAnnotation>>,
    plan: &mut RuntimeMetadataPlan,
) {
    for element in elements {
        match &element.data {
            ElementData::Paragraph(_) => collect_timers(
                element.ann.raw.as_str(),
                SectionIndexSource::from_annotation(&element.ann),
                outline_path,
                TimerContext::Paragraph,
                plan,
            ),
            ElementData::Drawer(drawer) => {
                if drawer.name.eq_ignore_ascii_case(FEEDSTATUS_DRAWER) {
                    collect_feed_status(element, section, plan);
                }
                collect_elements(&drawer.children, outline_path, section, plan);
            }
            ElementData::List(list) => {
                for item in &list.items {
                    let tag = objects_text(&item.tag);
                    collect_timers(
                        tag.as_str(),
                        SectionIndexSource::from_annotation(&item.ann),
                        outline_path,
                        TimerContext::ListItemTag,
                        plan,
                    );
                    collect_elements(&item.children, outline_path, section, plan);
                }
            }
            ElementData::Block(block) => {
                collect_elements(&block.children, outline_path, section, plan)
            }
            ElementData::FootnoteDef(footnote) => {
                collect_elements(&footnote.children, outline_path, section, plan);
            }
            ElementData::Inlinetask(task) => {
                collect_timers(
                    &task.raw_title,
                    SectionIndexSource::from_annotation(&element.ann),
                    outline_path,
                    TimerContext::Headline,
                    plan,
                );
                collect_elements(&task.children, outline_path, section, plan);
            }
            ElementData::Keyword(_)
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

fn collect_feed_status(
    element: &Element<ParsedAnnotation>,
    section: Option<&Section<ParsedAnnotation>>,
    plan: &mut RuntimeMetadataPlan,
) {
    let raw_body = drawer_body(element.ann.raw.as_str());
    let readable = feed_status_is_readable(raw_body.as_str());
    if !readable {
        plan.warnings.push(RuntimeMetadataWarning {
            kind: RuntimeMetadataWarningKind::UnreadableFeedStatus,
            message: "FEEDSTATUS drawer does not look like an Org Feed status list".to_string(),
        });
    }
    plan.feeds.push(FeedStatusRecord {
        source: SectionIndexSource::from_annotation(&element.ann),
        section_title: section
            .map(|section| section.raw_title.trim_end().to_string())
            .unwrap_or_default(),
        drawer: FeedStatusDrawerName::new(FEEDSTATUS_DRAWER),
        raw: raw_body.clone(),
        entry_count: feed_status_entry_count(raw_body.as_str()),
        readable,
    });
}

fn collect_timers(
    raw: &str,
    source: SectionIndexSource,
    outline_path: &[String],
    context: TimerContext,
    plan: &mut RuntimeMetadataPlan,
) {
    for stamp in timer_stamps(raw) {
        plan.timers.push(TimerRecord {
            source: source.clone(),
            outline_path: outline_path.to_vec(),
            context,
            raw: stamp.raw,
            total_seconds: stamp.total_seconds,
        });
    }
}

fn drawer_body(raw: &str) -> String {
    raw.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.eq_ignore_ascii_case(":FEEDSTATUS:") && !trimmed.eq_ignore_ascii_case(":END:")
        })
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn feed_status_is_readable(raw: &str) -> bool {
    let trimmed = raw.trim();
    trimmed.is_empty() || trimmed.starts_with('(')
}

fn feed_status_entry_count(raw: &str) -> usize {
    let bytes = raw.as_bytes();
    let mut count = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'(' {
            let mut next = index + 1;
            while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                next += 1;
            }
            if bytes.get(next) == Some(&b'"') {
                count += 1;
            }
        }
        index += 1;
    }
    count
}

struct TimerStamp {
    raw: String,
    total_seconds: i64,
}

fn timer_stamps(raw: &str) -> Vec<TimerStamp> {
    let bytes = raw.as_bytes();
    let mut stamps = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !is_timer_boundary_before(bytes, index) {
            index += 1;
            continue;
        }
        if let Some((stamp, next)) = parse_timer_at(bytes, index) {
            stamps.push(stamp);
            index = next;
        } else {
            index += 1;
        }
    }
    stamps
}

fn parse_timer_at(bytes: &[u8], index: usize) -> Option<(TimerStamp, usize)> {
    let mut cursor = index;
    let sign = match bytes.get(cursor) {
        Some(b'-') => {
            cursor += 1;
            -1
        }
        Some(b'+') => {
            cursor += 1;
            1
        }
        _ => 1,
    };
    let hour_start = cursor;
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
    }
    if cursor == hour_start || bytes.get(cursor) != Some(&b':') {
        return None;
    }
    let hours = std::str::from_utf8(&bytes[hour_start..cursor])
        .ok()?
        .parse::<i64>()
        .ok()?;
    cursor += 1;
    let minutes = parse_two_digits(bytes, cursor)?;
    if bytes.get(cursor + 2) != Some(&b':') {
        return None;
    }
    cursor += 3;
    let seconds = parse_two_digits(bytes, cursor)?;
    cursor += 2;
    if minutes > 59 || seconds > 59 || !is_timer_boundary_after(bytes, cursor) {
        return None;
    }
    let raw = std::str::from_utf8(&bytes[index..cursor]).ok()?.to_string();
    let total_seconds = sign * (hours * 3600 + minutes * 60 + seconds);
    Some((TimerStamp { raw, total_seconds }, cursor))
}

fn parse_two_digits(bytes: &[u8], index: usize) -> Option<i64> {
    let tens = *bytes.get(index)?;
    let ones = *bytes.get(index + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    Some(((tens - b'0') * 10 + (ones - b'0')) as i64)
}

fn is_timer_boundary_before(bytes: &[u8], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    !bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b':'
}

fn is_timer_boundary_after(bytes: &[u8], index: usize) -> bool {
    index >= bytes.len() || (!bytes[index].is_ascii_alphanumeric() && bytes[index] != b':')
}

fn original_id(section: &Section<ParsedAnnotation>) -> Option<(SectionIndexSource, String)> {
    section.properties.iter().find_map(|property| {
        property
            .key
            .eq_ignore_ascii_case(ORIGINAL_ID_PROPERTY)
            .then(|| {
                (
                    SectionIndexSource::from_annotation(&property.ann),
                    property.value.trim().to_string(),
                )
            })
    })
}

fn mobile_properties(properties: &[Property<ParsedAnnotation>]) -> Vec<MobileProperty> {
    properties
        .iter()
        .filter(|property| {
            property.key.eq_ignore_ascii_case(ORIGINAL_ID_PROPERTY)
                || property.key.to_ascii_uppercase().starts_with("MOBILE")
        })
        .map(|property| MobileProperty {
            source: SectionIndexSource::from_annotation(&property.ann),
            key: property.key.clone(),
            value: property.value.clone(),
        })
        .collect()
}

fn has_tag(tags: &[String], needle: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(needle))
}

fn title_file_link(
    objects: &[Object<ParsedAnnotation>],
    fallback_title: &str,
) -> Option<MobileIndexLink> {
    for object in objects {
        match &object.data {
            ObjectData::Link(link) => {
                let Some(file) = link.file.as_ref() else {
                    continue;
                };
                return Some(MobileIndexLink {
                    source: SectionIndexSource::from_annotation(&object.ann),
                    title: fallback_title.to_string(),
                    file: file.path.clone(),
                    description: objects_text(link.description_or_default()),
                });
            }
            ObjectData::Markup { children, .. } => {
                if let Some(link) = title_file_link(children, fallback_title) {
                    return Some(link);
                }
            }
            ObjectData::FootnoteRef { definition, .. } => {
                if let Some(link) = title_file_link(definition, fallback_title) {
                    return Some(link);
                }
            }
            ObjectData::Cloze { text, .. } => {
                if let Some(link) = title_file_link(text, fallback_title) {
                    return Some(link);
                }
            }
            ObjectData::Citation(_)
            | ObjectData::Plain(_)
            | ObjectData::LineBreak
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Timestamp(_)
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
    None
}

fn objects_text(objects: &[Object<ParsedAnnotation>]) -> String {
    objects.iter().map(object_text).collect::<Vec<_>>().join("")
}

fn object_text(object: &Object<ParsedAnnotation>) -> String {
    match &object.data {
        ObjectData::Plain(value)
        | ObjectData::Code(value)
        | ObjectData::Verbatim(value)
        | ObjectData::Entity(value)
        | ObjectData::LatexFragment(value)
        | ObjectData::Target(value)
        | ObjectData::RadioTarget(value)
        | ObjectData::StatisticCookie(value) => value.clone(),
        ObjectData::LineBreak => "\n".to_string(),
        ObjectData::Markup { children, .. } => objects_text(children),
        ObjectData::ExportSnippet { value, .. } => value.clone(),
        ObjectData::FootnoteRef { label, .. } => label.clone().unwrap_or_default(),
        ObjectData::Citation(citation) => citation
            .references
            .iter()
            .map(|reference| format!("@{}", reference.id))
            .collect::<Vec<_>>()
            .join(";"),
        ObjectData::Cloze { raw_text, .. } => raw_text.clone(),
        ObjectData::InlineCall { raw, .. }
        | ObjectData::InlineSrc { raw, .. }
        | ObjectData::Unknown { raw, .. } => raw.clone(),
        ObjectData::Link(link) => {
            let description = link.description_or_default();
            if description.is_empty() {
                link.path().to_string()
            } else {
                objects_text(description)
            }
        }
        ObjectData::Macro { name, arguments } => {
            if arguments.is_empty() {
                format!("{{{{{{{name}}}}}}}")
            } else {
                format!("{{{{{{{}({})}}}}}}", name, arguments.join(","))
            }
        }
        ObjectData::Timestamp(timestamp) => format!("{timestamp:?}"),
    }
}

fn split_words(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(str::to_string)
        .filter(|part| !part.is_empty())
        .collect()
}

fn runtime_boundaries() -> Vec<RuntimeMetadataBoundary> {
    vec![
        RuntimeMetadataBoundary {
            kind: RuntimeMetadataBoundaryKind::FeedNetworkUpdate,
            message: "RSS/Atom retrieval and feed item insertion remain outside orgize core"
                .to_string(),
        },
        RuntimeMetadataBoundary {
            kind: RuntimeMetadataBoundaryKind::TimerRuntimeState,
            message: "relative and countdown timer start/pause/stop state is editor runtime state"
                .to_string(),
        },
        RuntimeMetadataBoundary {
            kind: RuntimeMetadataBoundaryKind::MobileFilesystemSync,
            message:
                "Org Mobile push/pull, encryption, checksums, and file copying are not executed"
                    .to_string(),
        },
        RuntimeMetadataBoundary {
            kind: RuntimeMetadataBoundaryKind::OrgPersistCache,
            message:
                "org-persist cache registration and disk persistence are intentionally out of core"
                    .to_string(),
        },
    ]
}
