//! Project Scheme-parsed headline properties into attachment metadata.

use super::{
    AttachmentDirectory, AttachmentDirectorySource, AttachmentIdPathLayout, AttachmentState,
    ParsedAnnotation, Property,
};

pub(super) fn attachment_state(
    effective_tags: &[String],
    effective_properties: &[Property<ParsedAnnotation>],
) -> AttachmentState<ParsedAnnotation> {
    AttachmentState {
        has_attach_tag: effective_tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("ATTACH")),
        directory: attachment_directory(effective_properties),
    }
}

fn attachment_directory(
    properties: &[Property<ParsedAnnotation>],
) -> Option<AttachmentDirectory<ParsedAnnotation>> {
    let property = |key: &str| {
        properties
            .iter()
            .find(|property| property.key.eq_ignore_ascii_case(key))
            .filter(|property| !property.value.trim().is_empty())
    };
    property("DIR")
        .map(|property| AttachmentDirectory {
            ann: property.ann.clone(),
            source: AttachmentDirectorySource::DirProperty,
            path: property.value.trim().to_owned(),
        })
        .or_else(|| {
            property("ATTACH_DIR").map(|property| AttachmentDirectory {
                ann: property.ann.clone(),
                source: AttachmentDirectorySource::AttachDirProperty,
                path: property.value.trim().to_owned(),
            })
        })
        .or_else(|| property("ID").and_then(id_directory))
}

fn id_directory(
    property: &Property<ParsedAnnotation>,
) -> Option<AttachmentDirectory<ParsedAnnotation>> {
    let id = property.value.trim();
    let (layout, suffix) = if id.chars().count() > 2 {
        let (prefix, rest) = split_after_chars(id, 2)?;
        (AttachmentIdPathLayout::Uuid, format!("{prefix}/{rest}"))
    } else {
        let (prefix, _) = split_after_chars(id, 1)?;
        (
            AttachmentIdPathLayout::Fallback,
            format!("__/{prefix}/{id}"),
        )
    };
    Some(AttachmentDirectory {
        ann: property.ann.clone(),
        source: AttachmentDirectorySource::IdDerived {
            id: id.to_owned(),
            layout,
        },
        path: format!("data/{suffix}"),
    })
}

fn split_after_chars(value: &str, count: usize) -> Option<(&str, &str)> {
    let index = value
        .char_indices()
        .nth(count)
        .map(|(index, _)| index)
        .or_else(|| (value.chars().count() == count).then_some(value.len()))?;
    Some(value.split_at(index))
}
