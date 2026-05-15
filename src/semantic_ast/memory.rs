//! Org-native memory records and agent-facing memory snapshots.

use super::memory_model::{
    is_done_todo, AgentMemoryCard, AgentMemoryQuery, AgentMemorySnapshot, MemoryEvidence,
    MemoryEvidenceKind, MemoryLifecycleKind, MemoryLink, MemoryProperty, MemoryQuery, MemoryRecord,
    MemoryRecordState, MemorySource,
};
use super::model::{
    Citation, CiteReference, Document, Element, ElementData, Link, ListItem, Object, ObjectData,
    ParsedAnnotation, Property, Section, Timestamp,
};
use super::{lifecycle::section_lifecycle_records, LifecycleRecordKind};

impl Document<ParsedAnnotation> {
    /// Projects Org headlines into addressable memory records.
    ///
    /// This is an opt-in semantic view over official Org constructs. It does
    /// not mutate the parsed AST and it does not define project-specific Org
    /// syntax.
    pub fn memory_records(&self, query: &MemoryQuery) -> Vec<MemoryRecord> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_section_memory(section, query, &mut records);
        }
        records
    }

    /// Projects memory records into compact agent-facing cards.
    pub fn agent_memory_snapshot(&self, query: &AgentMemoryQuery) -> AgentMemorySnapshot {
        let cards = self
            .memory_records(&query.memory)
            .into_iter()
            .map(AgentMemoryCard::from_record)
            .collect();
        AgentMemorySnapshot { cards }
    }
}

fn collect_section_memory(
    section: &Section<ParsedAnnotation>,
    query: &MemoryQuery,
    records: &mut Vec<MemoryRecord>,
) {
    let state = classify_section(section);
    if section_matches_query(section, state, query) {
        records.push(memory_record(section, state));
    }

    for subsection in &section.subsections {
        collect_section_memory(subsection, query, records);
    }
}

fn memory_record(section: &Section<ParsedAnnotation>, state: MemoryRecordState) -> MemoryRecord {
    let mut evidence = Vec::new();
    let properties = section
        .properties
        .iter()
        .map(|property| memory_property(property, &mut evidence))
        .collect();

    if let Some(todo) = &section.todo {
        evidence.push(MemoryEvidence {
            source: MemorySource::from_annotation(&section.ann),
            kind: MemoryEvidenceKind::TodoState,
            value: todo.name.clone(),
        });
    }
    if section.archive.has_archive_tag {
        evidence.push(MemoryEvidence {
            source: MemorySource::from_annotation(&section.ann),
            kind: MemoryEvidenceKind::ArchiveTag,
            value: "ARCHIVE".to_string(),
        });
    }
    collect_archive_evidence(section, &mut evidence);
    collect_attachment_evidence(section, &mut evidence);
    collect_planning_evidence(section, &mut evidence);
    collect_lifecycle_evidence(section, &mut evidence);

    let mut links = Vec::new();
    collect_object_memory(&section.title, &mut evidence, &mut links);
    collect_element_memory(&section.children, &mut evidence, &mut links);

    MemoryRecord {
        source: MemorySource::from_annotation(&section.ann),
        state,
        level: section.level,
        title: section.raw_title.trim_end().to_string(),
        todo: section.todo.clone(),
        tags: section.tags.clone(),
        effective_tags: section.effective_tags.clone(),
        anchor: section.anchor.clone(),
        properties,
        evidence,
        links,
    }
}

fn memory_property(
    property: &Property<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
) -> MemoryProperty {
    evidence.push(MemoryEvidence {
        source: MemorySource::from_annotation(&property.ann),
        kind: MemoryEvidenceKind::Property {
            key: property.key.clone(),
        },
        value: property.value.clone(),
    });
    MemoryProperty {
        source: MemorySource::from_annotation(&property.ann),
        key: property.key.clone(),
        value: property.value.clone(),
    }
}

fn classify_section(section: &Section<ParsedAnnotation>) -> MemoryRecordState {
    if section.archive.archived {
        MemoryRecordState::Archived
    } else if is_done_todo(&section.todo) || section.planning.closed.is_some() {
        MemoryRecordState::Closed
    } else if section.todo.is_some()
        || section.planning.scheduled.is_some()
        || section.planning.deadline.is_some()
    {
        MemoryRecordState::Current
    } else {
        MemoryRecordState::Background
    }
}

fn collect_archive_evidence(
    section: &Section<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
) {
    if let Some(location) = &section.archive.property_location {
        evidence.push(MemoryEvidence {
            source: MemorySource::from_annotation(&location.ann),
            kind: MemoryEvidenceKind::ArchiveProperty,
            value: location.value.clone(),
        });
    } else if section.archive.archived {
        if let Some(location) = &section.archive.keyword_location {
            evidence.push(MemoryEvidence {
                source: MemorySource::from_annotation(&location.ann),
                kind: MemoryEvidenceKind::ArchiveLocation,
                value: location.value.clone(),
            });
        }
    }
}

fn collect_attachment_evidence(
    section: &Section<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
) {
    if section.attachment.has_attach_tag {
        evidence.push(MemoryEvidence {
            source: MemorySource::from_annotation(&section.ann),
            kind: MemoryEvidenceKind::AttachmentTag,
            value: "ATTACH".to_string(),
        });
    }
    if let Some(directory) = &section.attachment.directory {
        evidence.push(MemoryEvidence {
            source: MemorySource::from_annotation(&directory.ann),
            kind: MemoryEvidenceKind::AttachmentDirectory,
            value: directory.path.clone(),
        });
    }
}

fn section_matches_query(
    section: &Section<ParsedAnnotation>,
    state: MemoryRecordState,
    query: &MemoryQuery,
) -> bool {
    if section.is_comment && !query.include_comments {
        return false;
    }
    if state == MemoryRecordState::Closed && !query.include_closed {
        return false;
    }
    if state == MemoryRecordState::Archived && !query.include_archived {
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

fn collect_planning_evidence(
    section: &Section<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
) {
    if let Some(timestamp) = &section.planning.scheduled {
        evidence.push(timestamp_evidence(
            &section.ann,
            MemoryEvidenceKind::Scheduled,
            timestamp,
        ));
    }
    if let Some(timestamp) = &section.planning.deadline {
        evidence.push(timestamp_evidence(
            &section.ann,
            MemoryEvidenceKind::Deadline,
            timestamp,
        ));
    }
    if let Some(timestamp) = &section.planning.closed {
        evidence.push(timestamp_evidence(
            &section.ann,
            MemoryEvidenceKind::Closed,
            timestamp,
        ));
    }
}

fn collect_lifecycle_evidence(
    section: &Section<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
) {
    for record in section_lifecycle_records(section) {
        if matches!(record.kind, LifecycleRecordKind::MalformedLogbook { .. }) {
            continue;
        }
        evidence.push(MemoryEvidence {
            source: MemorySource::from_annotation(&record.ann),
            kind: MemoryEvidenceKind::Lifecycle(memory_lifecycle_kind(&record.kind)),
            value: record.raw,
        });
    }
}

fn memory_lifecycle_kind(kind: &LifecycleRecordKind) -> MemoryLifecycleKind {
    match kind {
        LifecycleRecordKind::StateChange { .. } => MemoryLifecycleKind::StateChange,
        LifecycleRecordKind::Note { .. } => MemoryLifecycleKind::Note,
        LifecycleRecordKind::Refile { .. } => MemoryLifecycleKind::Refile,
        LifecycleRecordKind::Reschedule { .. } => MemoryLifecycleKind::Reschedule,
        LifecycleRecordKind::Redeadline { .. } => MemoryLifecycleKind::Redeadline,
        LifecycleRecordKind::Clock { .. } => MemoryLifecycleKind::Clock,
        LifecycleRecordKind::MalformedLogbook { .. } => MemoryLifecycleKind::Note,
    }
}

fn collect_element_memory(
    elements: &[Element<ParsedAnnotation>],
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    for element in elements {
        collect_one_element_memory(element, evidence, links);
    }
}

fn collect_one_element_memory(
    element: &Element<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => collect_object_memory(objects, evidence, links),
        ElementData::Clock(clock) => collect_clock_memory(element, clock.raw.as_str(), evidence),
        ElementData::Drawer(drawer) => {
            collect_drawer_memory(element, drawer.name.as_str(), evidence);
            collect_element_memory(&drawer.children, evidence, links);
        }
        ElementData::List(list) => collect_list_memory(&list.items, evidence, links),
        ElementData::Table(table) => table.rows.iter().for_each(|row| {
            row.cells
                .iter()
                .for_each(|cell| collect_object_memory(&cell.objects, evidence, links));
        }),
        ElementData::Block(block) => collect_element_memory(&block.children, evidence, links),
        ElementData::FootnoteDef(footnote) => {
            collect_element_memory(&footnote.children, evidence, links);
        }
        ElementData::Inlinetask(task) => {
            collect_object_memory(&task.title, evidence, links);
            collect_element_memory(&task.children, evidence, links);
        }
        ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_clock_memory(
    element: &Element<ParsedAnnotation>,
    raw: &str,
    evidence: &mut Vec<MemoryEvidence>,
) {
    evidence.push(MemoryEvidence {
        source: MemorySource::from_annotation(&element.ann),
        kind: MemoryEvidenceKind::Clock,
        value: raw.to_string(),
    });
}

fn collect_drawer_memory(
    element: &Element<ParsedAnnotation>,
    name: &str,
    evidence: &mut Vec<MemoryEvidence>,
) {
    let kind = if name.eq_ignore_ascii_case("LOGBOOK") {
        MemoryEvidenceKind::Logbook
    } else {
        MemoryEvidenceKind::Drawer {
            name: name.to_string(),
        }
    };
    evidence.push(MemoryEvidence {
        source: MemorySource::from_annotation(&element.ann),
        kind,
        value: name.to_string(),
    });
}

fn collect_list_memory(
    items: &[ListItem<ParsedAnnotation>],
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    items
        .iter()
        .for_each(|item| collect_list_item_memory(item, evidence, links));
}

fn collect_list_item_memory(
    item: &ListItem<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    collect_object_memory(&item.tag, evidence, links);
    collect_element_memory(&item.children, evidence, links);
}

fn collect_object_memory(
    objects: &[Object<ParsedAnnotation>],
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    for object in objects {
        match &object.data {
            ObjectData::Timestamp(timestamp) => evidence.push(timestamp_evidence(
                &object.ann,
                MemoryEvidenceKind::Timestamp {
                    kind: timestamp.kind,
                },
                timestamp,
            )),
            ObjectData::Markup { children, .. } => collect_object_memory(children, evidence, links),
            ObjectData::Link(link) => collect_link_memory(object, link, evidence, links),
            ObjectData::FootnoteRef { definition, .. } => {
                collect_object_memory(definition, evidence, links);
            }
            ObjectData::Citation(citation) => collect_citation_memory(citation, evidence, links),
            ObjectData::Cloze { text, .. } => collect_object_memory(text, evidence, links),
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
}

fn collect_link_memory(
    object: &Object<ParsedAnnotation>,
    link: &Link<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    let description = if link.has_description() {
        link.raw_description.clone()
    } else {
        link.path().to_string()
    };
    let source = MemorySource::from_annotation(&object.ann);
    evidence.push(MemoryEvidence {
        source: source.clone(),
        kind: if link.attachment.is_some() {
            MemoryEvidenceKind::AttachmentLink
        } else {
            MemoryEvidenceKind::Link
        },
        value: link.path().to_string(),
    });
    links.push(MemoryLink {
        source,
        path: link.path().to_string(),
        description,
    });
    collect_object_memory(link.description_or_default(), evidence, links);
}

fn collect_citation_memory(
    citation: &Citation<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    collect_object_memory(&citation.prefix, evidence, links);
    collect_object_memory(&citation.suffix, evidence, links);
    for reference in &citation.references {
        collect_cite_reference_memory(reference, evidence, links);
    }
}

fn collect_cite_reference_memory(
    reference: &CiteReference<ParsedAnnotation>,
    evidence: &mut Vec<MemoryEvidence>,
    links: &mut Vec<MemoryLink>,
) {
    collect_object_memory(&reference.prefix, evidence, links);
    collect_object_memory(&reference.suffix, evidence, links);
}

fn timestamp_evidence(
    ann: &ParsedAnnotation,
    kind: MemoryEvidenceKind,
    timestamp: &Timestamp,
) -> MemoryEvidence {
    MemoryEvidence {
        source: MemorySource::from_annotation(ann),
        kind,
        value: timestamp.raw.clone(),
    }
}

fn has_tag(tags: &[String], needle: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(needle))
}
