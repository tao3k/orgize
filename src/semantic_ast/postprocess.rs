//! Semantic document post-processing that keeps source parsing lossless.

use std::collections::{HashMap, HashSet};

use super::{
    lifecycle::archive_location_from_property, ArchiveLocation, ArchiveState, AstMut, AstRef,
    AttachmentDirectory, AttachmentState, Diagnostic, DiagnosticKind, Document, ElementData,
    FootnoteDefinition, FootnoteEntry, LinkDescriptionState, LinkTarget, Object, ObjectData,
    ParsedAnnotation, Property, Section, TargetKind,
};

pub(super) fn finalize_document(document: &mut Document<ParsedAnnotation>) {
    assign_effective_tags_and_anchors(document);
    fill_target_aliases(document);
    fill_link_defaults(document);
    collect_and_resolve_footnotes(document);
}

fn assign_effective_tags_and_anchors(document: &mut Document<ParsedAnnotation>) {
    let mut known = HashSet::new();
    let filetags = document.filetags.clone();
    let properties = document.properties.clone();
    let archive_location = document.archive_locations.last().cloned();
    for section in &mut document.sections {
        assign_section_tags_anchor_properties_and_archive(
            section,
            &filetags,
            &properties,
            archive_location.as_ref(),
            &mut known,
        );
    }
}

fn assign_section_tags_anchor_properties_and_archive(
    section: &mut Section<ParsedAnnotation>,
    inherited_tags: &[String],
    inherited_properties: &[Property<ParsedAnnotation>],
    inherited_archive_location: Option<&ArchiveLocation<ParsedAnnotation>>,
    known: &mut HashSet<String>,
) {
    let mut effective = inherited_tags.to_vec();
    for tag in &section.tags {
        if !effective.iter().any(|existing| existing == tag) {
            effective.push(tag.clone());
        }
    }
    section.effective_tags = effective;
    section.effective_properties = merged_properties(inherited_properties, &section.properties);
    section.archive = archive_state(section, inherited_archive_location.cloned());
    section.attachment = attachment_state(section);

    let base = property_value(section, "CUSTOM_ID")
        .or_else(|| property_value(section, "ID"))
        .unwrap_or_else(|| slugify_title(&section.raw_title));
    section.anchor = (!base.is_empty()).then(|| unique_anchor(&base, known));

    let inherited_tags = section.effective_tags.clone();
    let inherited_properties = section.effective_properties.clone();
    let inherited_archive_location = section.archive.keyword_location.clone();
    for child in &mut section.subsections {
        assign_section_tags_anchor_properties_and_archive(
            child,
            &inherited_tags,
            &inherited_properties,
            inherited_archive_location.as_ref(),
            known,
        );
    }
}

fn archive_state(
    section: &Section<ParsedAnnotation>,
    keyword_location: Option<ArchiveLocation<ParsedAnnotation>>,
) -> ArchiveState<ParsedAnnotation> {
    let has_archive_tag = has_tag(&section.effective_tags, "ARCHIVE");
    let property_location = section
        .effective_properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case("ARCHIVE"))
        .map(archive_location_from_property);
    ArchiveState {
        archived: has_archive_tag,
        has_archive_tag,
        property_location,
        keyword_location,
    }
}

fn attachment_state(section: &Section<ParsedAnnotation>) -> AttachmentState<ParsedAnnotation> {
    let has_attach_tag = has_tag(&section.effective_tags, "ATTACH");
    let directory = attachment_directory(&section.effective_properties);
    AttachmentState {
        has_attach_tag,
        directory,
    }
}

fn attachment_directory(
    properties: &[Property<ParsedAnnotation>],
) -> Option<AttachmentDirectory<ParsedAnnotation>> {
    properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case("DIR"))
        .and_then(|property| {
            AttachmentDirectory::from_property_parts(
                property.ann.clone(),
                property.key.as_str(),
                property.value.as_str(),
            )
        })
        .or_else(|| {
            properties
                .iter()
                .find(|property| property.key.eq_ignore_ascii_case("ATTACH_DIR"))
                .and_then(|property| {
                    AttachmentDirectory::from_property_parts(
                        property.ann.clone(),
                        property.key.as_str(),
                        property.value.as_str(),
                    )
                })
        })
        .or_else(|| {
            properties
                .iter()
                .find(|property| property.key.eq_ignore_ascii_case("ID"))
                .and_then(|property| {
                    AttachmentDirectory::from_id_parts(
                        property.ann.clone(),
                        property.value.as_str(),
                    )
                })
        })
}

fn merged_properties(
    inherited: &[Property<ParsedAnnotation>],
    local: &[Property<ParsedAnnotation>],
) -> Vec<Property<ParsedAnnotation>> {
    let mut merged = inherited.to_vec();
    for property in local {
        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| existing.key.eq_ignore_ascii_case(&property.key))
        {
            *existing = property.clone();
        } else {
            merged.push(property.clone());
        }
    }
    merged
}

fn property_value(section: &Section<ParsedAnnotation>, key: &str) -> Option<String> {
    section
        .properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.clone())
        .filter(|value| !value.is_empty())
}

fn has_tag(tags: &[String], needle: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(needle))
}

fn unique_anchor(base: &str, known: &mut HashSet<String>) -> String {
    if known.insert(base.to_string()) {
        return base.to_string();
    }

    for suffix in 1usize.. {
        let candidate = format!("{base}-{suffix}");
        if known.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("infinite suffix iterator")
}

fn slugify_title(title: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(ch);
            pending_dash = false;
        } else if !slug.is_empty() {
            pending_dash = true;
        }
    }
    slug
}

fn fill_target_aliases(document: &mut Document<ParsedAnnotation>) {
    let aliases = target_aliases(document);
    for target in &mut document.targets {
        if let Some(alias) = aliases.get(&target.key) {
            target.alias = alias.clone();
        } else if matches!(target.kind, TargetKind::Target | TargetKind::RadioTarget) {
            target.alias = vec![plain_object(target.value.clone(), target.ann.clone())];
        }
    }
}

fn target_aliases(
    document: &Document<ParsedAnnotation>,
) -> HashMap<String, Vec<Object<ParsedAnnotation>>> {
    let mut aliases = HashMap::new();
    for section in &document.sections {
        collect_section_aliases(section, &mut aliases);
    }
    aliases
}

fn collect_section_aliases(
    section: &Section<ParsedAnnotation>,
    aliases: &mut HashMap<String, Vec<Object<ParsedAnnotation>>>,
) {
    let title = section.title.clone();
    let raw_title = section.raw_title.trim();
    if !raw_title.is_empty() {
        aliases
            .entry(raw_title.to_string())
            .or_insert(title.clone());
    }
    if let Some(custom_id) = property_value(section, "CUSTOM_ID") {
        aliases
            .entry(format!("#{custom_id}"))
            .or_insert(title.clone());
    }
    if let Some(id) = property_value(section, "ID") {
        aliases.entry(format!("id:{id}")).or_insert(title.clone());
    }
    for child in &section.subsections {
        collect_section_aliases(child, aliases);
    }
}

fn fill_link_defaults(document: &mut Document<ParsedAnnotation>) {
    let aliases = document
        .targets
        .iter()
        .map(|target| (target.key.clone(), target.alias.clone()))
        .collect::<HashMap<_, _>>();

    document.visit_mut(|node| {
        let AstMut::Object(object) = node else {
            return;
        };
        let ObjectData::Link(link) = &mut object.data else {
            return;
        };
        if !matches!(link.description_state, LinkDescriptionState::None) {
            return;
        }
        let LinkTarget::Internal(key) = &link.target else {
            return;
        };
        if let Some(alias) = aliases.get(key) {
            link.default_description = alias.clone();
        }
    });
}

fn collect_and_resolve_footnotes(document: &mut Document<ParsedAnnotation>) {
    let mut entries = standalone_footnotes(document);
    let known = known_footnote_labels(document, &entries);
    let mut diagnostics = Vec::new();
    let mut generated = 1usize;

    document.visit_mut(|node| {
        if let AstMut::Object(object) = node {
            resolve_footnote_object(object, &known, &mut generated, &mut diagnostics);
        }
    });

    entries.extend(inline_footnotes(document));
    document.diagnostics.extend(diagnostics);
    document.footnotes = entries;
}

fn standalone_footnotes(
    document: &Document<ParsedAnnotation>,
) -> Vec<FootnoteEntry<ParsedAnnotation>> {
    let mut entries = Vec::new();
    document.visit(|node| {
        let AstRef::Element(element) = node else {
            return;
        };
        let ElementData::FootnoteDef(definition) = &element.data else {
            return;
        };
        entries.push(FootnoteEntry {
            ann: element.ann.clone(),
            label: definition.label.clone(),
            definition: FootnoteDefinition::Standalone(definition.children.clone()),
        });
    });
    entries
}

fn known_footnote_labels(
    document: &Document<ParsedAnnotation>,
    entries: &[FootnoteEntry<ParsedAnnotation>],
) -> HashSet<String> {
    let mut labels = entries
        .iter()
        .map(|entry| entry.label.clone())
        .collect::<HashSet<_>>();
    document.visit(|node| {
        let AstRef::Object(object) = node else {
            return;
        };
        if let ObjectData::FootnoteRef {
            label: Some(label),
            definition,
            ..
        } = &object.data
        {
            if !definition.is_empty() {
                labels.insert(label.clone());
            }
        }
    });
    labels
}

fn resolve_footnote_object(
    object: &mut Object<ParsedAnnotation>,
    known: &HashSet<String>,
    generated: &mut usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let ObjectData::FootnoteRef {
        label,
        resolved_label,
        definition,
    } = &mut object.data
    else {
        return;
    };

    if definition.is_empty() {
        if let Some(label) = label {
            if known.contains(label) {
                *resolved_label = Some(label.clone());
            } else {
                diagnostics.push(Diagnostic {
                    range: object.ann.range,
                    kind: DiagnosticKind::Conversion,
                    message: format!("footnote reference `{label}` was not found"),
                });
            }
        }
        return;
    }

    let label = label.clone().unwrap_or_else(|| {
        let label = format!("fn-{generated}");
        *generated += 1;
        label
    });
    *resolved_label = Some(label);
}

fn inline_footnotes(document: &Document<ParsedAnnotation>) -> Vec<FootnoteEntry<ParsedAnnotation>> {
    let mut entries = Vec::new();
    document.visit(|node| {
        let AstRef::Object(object) = node else {
            return;
        };
        if let ObjectData::FootnoteRef {
            resolved_label: Some(label),
            definition,
            ..
        } = &object.data
        {
            if !definition.is_empty() {
                entries.push(FootnoteEntry {
                    ann: object.ann.clone(),
                    label: label.clone(),
                    definition: FootnoteDefinition::Inline(definition.clone()),
                });
            }
        }
    });
    entries
}

fn plain_object(value: String, ann: ParsedAnnotation) -> Object<ParsedAnnotation> {
    Object {
        ann,
        data: ObjectData::Plain(value),
    }
}
