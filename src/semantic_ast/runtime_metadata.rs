//! Non-executing projections for runtime-adjacent Org metadata.

use std::collections::HashSet;

use super::aot_drawer_projection::drawer_body;

use super::{
    AstRef, Document, Element, ElementData, FeedStatusDrawerName, FeedStatusRecord,
    MobileFlaggedSection, MobileIndexLink, MobileOriginalId, MobilePriorityDeclaration,
    MobileProperty, MobileReadonlyKeyword, Object, ObjectData, ParsedAnnotation, Property,
    RuntimeMetadataBoundary, RuntimeMetadataBoundaryKind, RuntimeMetadataPlan,
    RuntimeMetadataWarning, RuntimeMetadataWarningKind, Section, SectionIndexSource, TimerContext,
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
    let mut seen = HashSet::new();
    document.fold((), |(), node| {
        if let AstRef::Keyword(keyword) = node {
            let source_start = u32::from(keyword.ann.range.start());
            if !seen.insert(source_start) {
                return;
            }
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
    });
    plan.mobile
        .readonly
        .sort_by_key(|entry| entry.source.range_start);
    plan.mobile
        .all_priorities
        .sort_by_key(|entry| entry.source.range_start);
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
    let body = drawer_body(&element.ann);
    let row = super::org_native_values::optional("feed-status", body).expect("native feed row");
    let [raw_body, readable, count]: [String; 3] = row.try_into().expect("native feed arity");
    let readable = match readable.as_str() {
        "true" => true,
        "false" => false,
        _ => panic!("native feed boolean"),
    };
    let entry_count = count.parse().expect("native feed count");
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
        entry_count,
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

struct TimerStamp {
    raw: String,
    total_seconds: i64,
}

fn timer_stamps(raw: &str) -> Vec<TimerStamp> {
    super::org_native_values::rows("timer-stamps", &[raw])
        .into_iter()
        .map(|row| {
            let [raw, seconds]: [String; 2] = row.try_into().expect("native timer arity");
            TimerStamp {
                raw,
                total_seconds: seconds.parse().expect("native timer seconds"),
            }
        })
        .collect()
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
    super::org_native_values::words(value)
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
