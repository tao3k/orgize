//! Official Org special-property projection helpers.

use super::model::{
    Citation, CiteReference, Element, ElementData, Link, ListItem, Object, ObjectData, Section,
};
use super::timestamp_model::TimestampKind;

/// Optional caller context for special-property projection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpecialPropertyContext<'a> {
    pub(crate) category: Option<&'a str>,
    pub(crate) file: Option<&'a str>,
}

impl<'a> SpecialPropertyContext<'a> {
    pub(crate) fn new(category: Option<&'a str>, file: Option<&'a str>) -> Self {
        Self { category, file }
    }
}

/// One computed official Org special property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SpecialProperty {
    pub(crate) name: &'static str,
    pub(crate) value: String,
}

const SUPPORTED_SPECIAL_PROPERTIES: &[&str] = &[
    "ITEM",
    "TODO",
    "LEVEL",
    "PRIORITY",
    "TAGS",
    "ALLTAGS",
    "CATEGORY",
    "FILE",
    "SCHEDULED",
    "DEADLINE",
    "CLOSED",
    "TIMESTAMP",
    "TIMESTAMP_IA",
];

pub(crate) fn special_properties<A>(
    section: &Section<A>,
    context: SpecialPropertyContext<'_>,
) -> Vec<SpecialProperty> {
    SUPPORTED_SPECIAL_PROPERTIES
        .iter()
        .filter_map(|name| {
            special_property_value(section, context, name)
                .map(|value| SpecialProperty { name, value })
        })
        .collect()
}

pub(crate) fn special_property_value<A>(
    section: &Section<A>,
    context: SpecialPropertyContext<'_>,
    key: &str,
) -> Option<String> {
    if key.eq_ignore_ascii_case("ITEM") {
        return Some(section.raw_title.trim_end().to_string());
    }
    if key.eq_ignore_ascii_case("TODO") {
        return section.todo.as_ref().map(|todo| todo.name.clone());
    }
    if key.eq_ignore_ascii_case("LEVEL") {
        return Some(section.level.to_string());
    }
    if key.eq_ignore_ascii_case("PRIORITY") {
        return Some(section.priority.effective_text());
    }
    if key.eq_ignore_ascii_case("TAGS") {
        return Some(tag_string(&section.tags));
    }
    if key.eq_ignore_ascii_case("ALLTAGS") {
        return Some(tag_string(&section.effective_tags));
    }
    if key.eq_ignore_ascii_case("CATEGORY") {
        return context.category.map(ToOwned::to_owned);
    }
    if key.eq_ignore_ascii_case("FILE") {
        return context.file.map(ToOwned::to_owned);
    }
    if key.eq_ignore_ascii_case("SCHEDULED") {
        return section
            .planning
            .scheduled
            .as_ref()
            .map(|timestamp| timestamp.raw.clone());
    }
    if key.eq_ignore_ascii_case("DEADLINE") {
        return section
            .planning
            .deadline
            .as_ref()
            .map(|timestamp| timestamp.raw.clone());
    }
    if key.eq_ignore_ascii_case("CLOSED") {
        return section
            .planning
            .closed
            .as_ref()
            .map(|timestamp| timestamp.raw.clone());
    }
    if key.eq_ignore_ascii_case("TIMESTAMP") {
        return first_entry_timestamp(section, TimestampKind::Active);
    }
    if key.eq_ignore_ascii_case("TIMESTAMP_IA") {
        return first_entry_timestamp(section, TimestampKind::Inactive);
    }
    None
}

pub(crate) fn tag_string(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(":{}:", tags.join(":"))
    }
}

fn first_entry_timestamp<A>(section: &Section<A>, kind: TimestampKind) -> Option<String> {
    let mut found = None;
    collect_timestamp_in_objects(&section.title, kind, &mut found);
    if found.is_some() {
        return found;
    }
    collect_timestamp_in_elements(&section.children, kind, &mut found);
    found
}

fn collect_timestamp_in_elements<A>(
    elements: &[Element<A>],
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    for element in elements {
        if found.is_some() {
            return;
        }
        collect_timestamp_in_element(element, kind, found);
    }
}

fn collect_timestamp_in_element<A>(
    element: &Element<A>,
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    match &element.data {
        ElementData::Paragraph(objects) => collect_timestamp_in_objects(objects, kind, found),
        ElementData::Drawer(drawer) => collect_timestamp_in_elements(&drawer.children, kind, found),
        ElementData::List(list) => collect_timestamp_in_list_items(&list.items, kind, found),
        ElementData::Table(table) => collect_timestamp_in_table_cells(table, kind, found),
        ElementData::Block(block) => collect_timestamp_in_elements(&block.children, kind, found),
        ElementData::FootnoteDef(footnote) => {
            collect_timestamp_in_elements(&footnote.children, kind, found);
        }
        ElementData::Inlinetask(task) => collect_timestamp_in_inlinetask(task, kind, found),
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

fn collect_timestamp_in_table_cells<A>(
    table: &super::model::Table<A>,
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    for cell in table.rows.iter().flat_map(|row| &row.cells) {
        collect_timestamp_in_objects(&cell.objects, kind, found);
        if found.is_some() {
            return;
        }
    }
}

fn collect_timestamp_in_inlinetask<A>(
    task: &super::model::Inlinetask<A>,
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    collect_timestamp_in_objects(&task.title, kind, found);
    collect_timestamp_in_elements(&task.children, kind, found);
}

fn collect_timestamp_in_list_items<A>(
    items: &[ListItem<A>],
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    for item in items {
        collect_timestamp_in_objects(&item.tag, kind, found);
        collect_timestamp_in_elements(&item.children, kind, found);
        if found.is_some() {
            return;
        }
    }
}

fn collect_timestamp_in_objects<A>(
    objects: &[Object<A>],
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    for object in objects {
        if found.is_some() {
            return;
        }
        match &object.data {
            ObjectData::Timestamp(timestamp) if timestamp.kind == kind => {
                *found = Some(timestamp.raw.clone());
            }
            ObjectData::Markup { children, .. } => {
                collect_timestamp_in_objects(children, kind, found);
            }
            ObjectData::Link(link) => collect_timestamp_in_link(link, kind, found),
            ObjectData::FootnoteRef { definition, .. } => {
                collect_timestamp_in_objects(definition, kind, found);
            }
            ObjectData::Citation(citation) => collect_timestamp_in_citation(citation, kind, found),
            ObjectData::Cloze { text, .. } => collect_timestamp_in_objects(text, kind, found),
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
            | ObjectData::Target(_)
            | ObjectData::RadioTarget(_)
            | ObjectData::Macro { .. }
            | ObjectData::StatisticCookie(_)
            | ObjectData::Unknown { .. } => {}
        }
    }
}

fn collect_timestamp_in_link<A>(link: &Link<A>, kind: TimestampKind, found: &mut Option<String>) {
    collect_timestamp_in_objects(link.description_or_default(), kind, found);
}

fn collect_timestamp_in_citation<A>(
    citation: &Citation<A>,
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    collect_timestamp_in_objects(&citation.prefix, kind, found);
    collect_timestamp_in_objects(&citation.suffix, kind, found);
    for reference in &citation.references {
        collect_timestamp_in_cite_reference(reference, kind, found);
        if found.is_some() {
            return;
        }
    }
}

fn collect_timestamp_in_cite_reference<A>(
    reference: &CiteReference<A>,
    kind: TimestampKind,
    found: &mut Option<String>,
) {
    collect_timestamp_in_objects(&reference.prefix, kind, found);
    collect_timestamp_in_objects(&reference.suffix, kind, found);
}

pub(crate) fn timestamp_sort_key(value: &str) -> Option<(u16, u8, u8, u8, u8)> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index + 10 <= bytes.len() {
        if let Some(key) = timestamp_sort_key_at(value, index) {
            return Some(key);
        }
        index += 1;
    }
    None
}

fn timestamp_sort_key_at(value: &str, index: usize) -> Option<(u16, u8, u8, u8, u8)> {
    let bytes = value.as_bytes();
    if index + 10 > bytes.len()
        || !bytes[index].is_ascii_digit()
        || !bytes[index + 1].is_ascii_digit()
        || !bytes[index + 2].is_ascii_digit()
        || !bytes[index + 3].is_ascii_digit()
        || bytes[index + 4] != b'-'
        || !bytes[index + 5].is_ascii_digit()
        || !bytes[index + 6].is_ascii_digit()
        || bytes[index + 7] != b'-'
        || !bytes[index + 8].is_ascii_digit()
        || !bytes[index + 9].is_ascii_digit()
    {
        return None;
    }
    let year = value[index..index + 4].parse().ok()?;
    let month = value[index + 5..index + 7].parse().ok()?;
    let day = value[index + 8..index + 10].parse().ok()?;
    let (hour, minute) = timestamp_time_at(value, index + 10).unwrap_or((0, 0));
    Some((year, month, day, hour, minute))
}

fn timestamp_time_at(value: &str, start: usize) -> Option<(u8, u8)> {
    let bytes = value.as_bytes();
    let mut index = start;
    while index + 5 <= bytes.len() {
        if bytes[index].is_ascii_digit()
            && bytes[index + 1].is_ascii_digit()
            && bytes[index + 2] == b':'
            && bytes[index + 3].is_ascii_digit()
            && bytes[index + 4].is_ascii_digit()
        {
            return Some((
                value[index..index + 2].parse().ok()?,
                value[index + 3..index + 5].parse().ok()?,
            ));
        }
        if matches!(bytes[index], b'>' | b']') {
            return None;
        }
        index += 1;
    }
    None
}
