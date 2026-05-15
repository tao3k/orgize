//! Source-grounded section records for document-local index builders.

use super::lifecycle::section_lifecycle_records;
use super::section_index_model::{
    SectionIndexArchive, SectionIndexAttachment, SectionIndexAttachmentDirectory,
    SectionIndexCategory, SectionIndexLifecycleRecord, SectionIndexLink, SectionIndexProperty,
    SectionIndexRecord, SectionIndexSource, SectionIndexTarget, SectionIndexTextSlice,
};
use super::{
    Document, Element, ElementData, Link, ListItem, Object, ObjectData, ParsedAnnotation, Property,
    Section, TargetKind,
};

impl Document<ParsedAnnotation> {
    /// Projects sections into document-local records for downstream indexers.
    ///
    /// This API does not perform search, ranking, or cross-document resolution.
    /// It preserves official Org syntax and source evidence so callers such as
    /// Wendao can build their own retrieval indexes.
    pub fn section_index_records(&self) -> Vec<SectionIndexRecord> {
        let mut records = Vec::new();
        let document_category = document_category(self);
        for section in &self.sections {
            collect_section_index(section, Vec::new(), document_category.clone(), &mut records);
        }
        records
    }
}

fn collect_section_index(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    inherited_category: Option<SectionIndexCategory>,
    records: &mut Vec<SectionIndexRecord>,
) {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path;
    outline_path.push(title.clone());
    let category = section_category(section).or(inherited_category);

    records.push(section_index_record(
        section,
        outline_path.clone(),
        title,
        category.clone(),
    ));

    for subsection in &section.subsections {
        collect_section_index(subsection, outline_path.clone(), category.clone(), records);
    }
}

fn section_index_record(
    section: &Section<ParsedAnnotation>,
    outline_path: Vec<String>,
    title: String,
    category: Option<SectionIndexCategory>,
) -> SectionIndexRecord {
    let mut links = Vec::new();
    let mut targets = section_targets(section);
    collect_object_index_metadata(&section.title, &mut links, &mut targets);
    collect_element_index_metadata(&section.children, &mut links, &mut targets);

    SectionIndexRecord {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path,
        level: section.level,
        title,
        body: body_slices(&section.children),
        todo: section.todo.clone(),
        priority: section.priority.clone(),
        category,
        tags: section.tags.clone(),
        effective_tags: section.effective_tags.clone(),
        properties: section.properties.iter().map(section_property).collect(),
        effective_properties: section
            .effective_properties
            .iter()
            .map(section_property)
            .collect(),
        planning: section.planning.clone(),
        is_comment: section.is_comment,
        archive: section_archive(section),
        attachment: section_attachment(section),
        links,
        targets,
        lifecycle: section_lifecycle_records(section)
            .into_iter()
            .map(|record| SectionIndexLifecycleRecord {
                source: SectionIndexSource::from_annotation(&record.ann),
                kind: record.kind,
                raw: record.raw,
            })
            .collect(),
    }
}

fn body_slices(elements: &[Element<ParsedAnnotation>]) -> Vec<SectionIndexTextSlice> {
    elements
        .iter()
        .filter(|element| !matches!(element.data, ElementData::PropertyDrawer(_)))
        .filter_map(|element| {
            let text = element.ann.raw.trim().to_string();
            (!text.is_empty()).then(|| SectionIndexTextSlice {
                source: SectionIndexSource::from_annotation(&element.ann),
                text,
            })
        })
        .collect()
}

fn section_property(property: &Property<ParsedAnnotation>) -> SectionIndexProperty {
    SectionIndexProperty {
        source: SectionIndexSource::from_annotation(&property.ann),
        key: property.key.clone(),
        value: property.value.clone(),
    }
}

fn section_archive(section: &Section<ParsedAnnotation>) -> SectionIndexArchive {
    SectionIndexArchive {
        archived: section.archive.archived,
        has_archive_tag: section.archive.has_archive_tag,
        location: section
            .archive
            .location()
            .map(|location| location.value.clone()),
    }
}

fn section_attachment(section: &Section<ParsedAnnotation>) -> SectionIndexAttachment {
    SectionIndexAttachment {
        has_attach_tag: section.attachment.has_attach_tag,
        directory: section.attachment.directory.as_ref().map(|directory| {
            SectionIndexAttachmentDirectory {
                source: directory.source.clone(),
                path: directory.path.clone(),
            }
        }),
    }
}

fn section_targets(section: &Section<ParsedAnnotation>) -> Vec<SectionIndexTarget> {
    let mut targets = Vec::new();
    if !section.raw_title.trim().is_empty() {
        targets.push(SectionIndexTarget {
            source: SectionIndexSource::from_annotation(&section.ann),
            kind: TargetKind::Headline,
            key: section.raw_title.trim().to_string(),
            value: section.raw_title.trim().to_string(),
        });
    }
    for property in &section.properties {
        if property.value.trim().is_empty() {
            continue;
        }
        if property.key.eq_ignore_ascii_case("CUSTOM_ID") {
            targets.push(SectionIndexTarget {
                source: SectionIndexSource::from_annotation(&property.ann),
                kind: TargetKind::CustomId,
                key: format!("#{}", property.value),
                value: property.value.clone(),
            });
        } else if property.key.eq_ignore_ascii_case("ID") {
            targets.push(SectionIndexTarget {
                source: SectionIndexSource::from_annotation(&property.ann),
                kind: TargetKind::Id,
                key: format!("id:{}", property.value),
                value: property.value.clone(),
            });
        }
    }
    targets
}

fn collect_element_index_metadata(
    elements: &[Element<ParsedAnnotation>],
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    for element in elements {
        collect_one_element_index_metadata(element, links, targets);
    }
}

fn collect_one_element_index_metadata(
    element: &Element<ParsedAnnotation>,
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => collect_object_index_metadata(objects, links, targets),
        ElementData::Drawer(drawer) => {
            collect_element_index_metadata(&drawer.children, links, targets)
        }
        ElementData::List(list) => collect_list_index_metadata(&list.items, links, targets),
        ElementData::Table(table) => collect_table_index_metadata(table, links, targets),
        ElementData::Block(block) => {
            collect_block_code_ref_targets(element, block, targets);
            collect_element_index_metadata(&block.children, links, targets);
        }
        ElementData::FootnoteDef(footnote) => {
            targets.push(SectionIndexTarget {
                source: SectionIndexSource::from_annotation(&element.ann),
                kind: TargetKind::FootnoteDefinition,
                key: format!("fn:{}", footnote.label),
                value: footnote.label.clone(),
            });
            collect_element_index_metadata(&footnote.children, links, targets);
        }
        ElementData::Inlinetask(task) => {
            collect_object_index_metadata(&task.title, links, targets);
            collect_element_index_metadata(&task.children, links, targets);
        }
        ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::TableEl { .. }
        | ElementData::Comment(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_list_index_metadata(
    items: &[ListItem<ParsedAnnotation>],
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    for item in items {
        collect_list_item_index_metadata(item, links, targets);
    }
}

fn collect_table_index_metadata(
    table: &super::Table<ParsedAnnotation>,
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    for cell in table.rows.iter().flat_map(|row| &row.cells) {
        collect_object_index_metadata(&cell.objects, links, targets);
    }
}

fn collect_block_code_ref_targets(
    element: &Element<ParsedAnnotation>,
    block: &super::Block<ParsedAnnotation>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    for code_ref in &block.code_refs {
        targets.push(SectionIndexTarget {
            source: SectionIndexSource::from_annotation(&element.ann),
            kind: TargetKind::CodeRef,
            key: format!("coderef:{}", code_ref.name),
            value: code_ref.name.clone(),
        });
    }
}

fn collect_list_item_index_metadata(
    item: &ListItem<ParsedAnnotation>,
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    collect_object_index_metadata(&item.tag, links, targets);
    collect_element_index_metadata(&item.children, links, targets);
}

fn collect_object_index_metadata(
    objects: &[Object<ParsedAnnotation>],
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    for object in objects {
        match &object.data {
            ObjectData::Markup { children, .. } => {
                collect_object_index_metadata(children, links, targets);
            }
            ObjectData::Link(link) => collect_link_index_metadata(object, link, links, targets),
            ObjectData::FootnoteRef { definition, .. } => {
                collect_object_index_metadata(definition, links, targets);
            }
            ObjectData::Citation(citation) => {
                collect_object_index_metadata(&citation.prefix, links, targets);
                collect_object_index_metadata(&citation.suffix, links, targets);
                for reference in &citation.references {
                    collect_object_index_metadata(&reference.prefix, links, targets);
                    collect_object_index_metadata(&reference.suffix, links, targets);
                }
            }
            ObjectData::Cloze { text, .. } => collect_object_index_metadata(text, links, targets),
            ObjectData::Target(value) => targets.push(SectionIndexTarget {
                source: SectionIndexSource::from_annotation(&object.ann),
                kind: TargetKind::Target,
                key: value.clone(),
                value: value.clone(),
            }),
            ObjectData::RadioTarget(value) => targets.push(SectionIndexTarget {
                source: SectionIndexSource::from_annotation(&object.ann),
                kind: TargetKind::RadioTarget,
                key: value.clone(),
                value: value.clone(),
            }),
            ObjectData::Plain(_)
            | ObjectData::LineBreak
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Timestamp(_)
            | ObjectData::Entity(_)
            | ObjectData::LatexFragment(_)
            | ObjectData::ExportSnippet { .. }
            | ObjectData::InlineCall { .. }
            | ObjectData::InlineSrc { .. }
            | ObjectData::Macro { .. }
            | ObjectData::StatisticCookie(_)
            | ObjectData::Unknown { .. } => {}
        }
    }
}

fn collect_link_index_metadata(
    object: &Object<ParsedAnnotation>,
    link: &Link<ParsedAnnotation>,
    links: &mut Vec<SectionIndexLink>,
    targets: &mut Vec<SectionIndexTarget>,
) {
    let description = if link.has_description() {
        link.raw_description.clone()
    } else {
        link.path().to_string()
    };
    links.push(SectionIndexLink {
        source: SectionIndexSource::from_annotation(&object.ann),
        path: link.path().to_string(),
        description,
        search: link.search.clone(),
        attachment: link.attachment.as_deref().cloned(),
    });
    collect_object_index_metadata(link.description_or_default(), links, targets);
}

fn document_category(document: &Document<ParsedAnnotation>) -> Option<SectionIndexCategory> {
    property_category(&document.properties).or_else(|| keyword_category(&document.children))
}

fn section_category(section: &Section<ParsedAnnotation>) -> Option<SectionIndexCategory> {
    property_category(&section.properties)
}

fn property_category(properties: &[Property<ParsedAnnotation>]) -> Option<SectionIndexCategory> {
    properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case("CATEGORY"))
        .map(|property| property.value.trim())
        .filter(|value| !value.is_empty())
        .map(SectionIndexCategory::new)
}

fn keyword_category(elements: &[Element<ParsedAnnotation>]) -> Option<SectionIndexCategory> {
    elements
        .iter()
        .filter_map(|element| match &element.data {
            ElementData::Keyword(keyword) if keyword.key.eq_ignore_ascii_case("CATEGORY") => {
                Some(keyword.value.trim())
            }
            _ => None,
        })
        .find(|value| !value.is_empty())
        .map(SectionIndexCategory::new)
}
