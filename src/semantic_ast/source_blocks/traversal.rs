//! Source-block side-table projection for Babel/tangle-aware consumers.

use crate::ast::{
    BlockKind, Document, Element, ElementData, Keyword, ListItem, Object, ObjectData,
    ParsedAnnotation, Property, Section, SourceBlockRecord, SourceBlockResult,
    SourceBlockResultKind, SourceBlockSource,
};

use super::{SourceBlockSyntaxRecord, projection::SourceBlockProjection};

impl Document<ParsedAnnotation> {
    /// Projects source blocks into a document-local side table.
    ///
    /// This does not execute Babel blocks or resolve tangle destinations. It
    /// preserves ordinary Org metadata so downstream indexers can decide how to
    /// store, lint, or display source-block evidence.
    pub fn source_block_records(&self) -> Vec<SourceBlockRecord> {
        let mut records = Vec::new();
        collect_source_block_records_in_elements(&self.children, &self.properties, &mut records);
        for section in &self.sections {
            collect_source_block_records_in_section(section, &mut records);
        }
        records
    }
    /// Projects only source syntax; does not compute Babel execution policies.
    pub(crate) fn source_block_syntax_records(&self) -> Vec<SourceBlockSyntaxRecord> {
        let mut records = Vec::new();
        collect_source_block_records_in_elements(&self.children, &self.properties, &mut records);
        for section in &self.sections {
            collect_source_block_records_in_section(section, &mut records);
        }
        records
    }
}

fn collect_source_block_records_in_section(
    section: &Section<ParsedAnnotation>,
    records: &mut impl SourceBlockProjection,
) {
    collect_source_block_records_in_elements(
        &section.children,
        &section.effective_properties,
        records,
    );
    for subsection in &section.subsections {
        collect_source_block_records_in_section(subsection, records);
    }
}

fn collect_source_block_records_in_elements(
    elements: &[Element<ParsedAnnotation>],
    properties: &[Property<ParsedAnnotation>],
    records: &mut impl SourceBlockProjection,
) {
    for (index, element) in elements.iter().enumerate() {
        if let ElementData::Block(block) = &element.data {
            if block.kind == BlockKind::Source {
                records.block(element, block, properties, elements.get(index + 1));
            }
            collect_source_block_records_in_elements(&block.children, properties, records);
        }
        collect_inline_source_records(element, properties, records);
        collect_nested_source_block_records(element, properties, records);
    }
}

fn collect_nested_source_block_records(
    element: &Element<ParsedAnnotation>,
    properties: &[Property<ParsedAnnotation>],
    records: &mut impl SourceBlockProjection,
) {
    match &element.data {
        ElementData::Drawer(drawer) => {
            collect_source_block_records_in_elements(&drawer.children, properties, records)
        }
        ElementData::List(list) => {
            collect_source_block_records_in_list_items(&list.items, properties, records)
        }
        ElementData::FootnoteDef(footnote) => {
            collect_source_block_records_in_elements(&footnote.children, properties, records);
        }
        ElementData::Inlinetask(task) => {
            let scoped_properties = merged_properties(properties, &task.properties);
            collect_source_block_records_in_elements(&task.children, &scoped_properties, records);
        }
        ElementData::Paragraph(_)
        | ElementData::Table(_)
        | ElementData::Keyword(_)
        | ElementData::BabelCall(_)
        | ElementData::Clock(_)
        | ElementData::PropertyDrawer(_)
        | ElementData::TableEl { .. }
        | ElementData::Block(_)
        | ElementData::Comment(_)
        | ElementData::DiarySexp(_)
        | ElementData::FixedWidth(_)
        | ElementData::Rule
        | ElementData::LatexEnvironment(_)
        | ElementData::Unknown { .. } => {}
    }
}

fn collect_source_block_records_in_list_items(
    items: &[ListItem<ParsedAnnotation>],
    properties: &[Property<ParsedAnnotation>],
    records: &mut impl SourceBlockProjection,
) {
    for item in items {
        collect_source_block_records_in_elements(&item.children, properties, records);
    }
}

fn collect_inline_source_records(
    element: &Element<ParsedAnnotation>,
    properties: &[Property<ParsedAnnotation>],
    records: &mut impl SourceBlockProjection,
) {
    match &element.data {
        ElementData::Paragraph(objects) => {
            collect_inline_source_records_in_objects(objects, properties, records)
        }
        ElementData::Table(table) => {
            for row in &table.rows {
                for cell in &row.cells {
                    collect_inline_source_records_in_objects(&cell.objects, properties, records);
                }
            }
        }
        ElementData::List(list) => {
            for item in &list.items {
                collect_inline_source_records_in_objects(&item.tag, properties, records);
            }
        }
        ElementData::Inlinetask(task) => {
            let scoped_properties = merged_properties(properties, &task.properties);
            collect_inline_source_records_in_objects(&task.title, &scoped_properties, records);
        }
        _ => {}
    }
}

fn collect_inline_source_records_in_objects(
    objects: &[Object<ParsedAnnotation>],
    properties: &[Property<ParsedAnnotation>],
    records: &mut impl SourceBlockProjection,
) {
    for (index, object) in objects.iter().enumerate() {
        match &object.data {
            ObjectData::InlineSrc { .. } => {
                records.inline(object, properties, objects.get(index + 1));
            }
            ObjectData::Markup { children, .. } => {
                collect_inline_source_records_in_objects(children, properties, records);
            }
            ObjectData::FootnoteRef { definition, .. } => {
                collect_inline_source_records_in_objects(definition, properties, records);
            }
            ObjectData::Citation(citation) => {
                collect_inline_source_records_in_objects(&citation.prefix, properties, records);
                collect_inline_source_records_in_objects(&citation.suffix, properties, records);
                for reference in &citation.references {
                    collect_inline_source_records_in_objects(
                        &reference.prefix,
                        properties,
                        records,
                    );
                    collect_inline_source_records_in_objects(
                        &reference.suffix,
                        properties,
                        records,
                    );
                }
            }
            ObjectData::Link(link) => {
                collect_inline_source_records_in_objects(&link.description, properties, records);
            }
            ObjectData::Cloze { text, .. } => {
                collect_inline_source_records_in_objects(text, properties, records);
            }
            ObjectData::Plain(_)
            | ObjectData::LineBreak
            | ObjectData::Code(_)
            | ObjectData::Verbatim(_)
            | ObjectData::Timestamp(_)
            | ObjectData::Entity(_)
            | ObjectData::LatexFragment(_)
            | ObjectData::ExportSnippet { .. }
            | ObjectData::InlineCall { .. }
            | ObjectData::Target(_)
            | ObjectData::RadioTarget(_)
            | ObjectData::Macro { .. }
            | ObjectData::StatisticCookie(_)
            | ObjectData::Unknown { .. } => {}
        }
    }
}

pub(super) fn affiliated_keyword_value(
    keywords: &[Keyword<ParsedAnnotation>],
    key: &str,
) -> Option<String> {
    keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case(key))
        .map(|keyword| keyword.value.trim().to_string())
        .filter(|value| !value.is_empty())
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

pub(super) fn source_block_result_from_element(
    element: &Element<ParsedAnnotation>,
) -> Option<SourceBlockResult> {
    let keyword = element
        .affiliated_keywords
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case("RESULTS"))?;
    // The result body is a separate AOT element; its affiliated keyword is
    // part of the replaceable result, not part of the preceding source block.
    let mut source = SourceBlockSource::from_annotation(&element.ann);
    source.start = keyword.ann.start;
    source.range_start = keyword.ann.range.start().into();
    Some(SourceBlockResult {
        source,
        kind: SourceBlockResultKind::Keyword,
        hash: keyword
            .optional
            .clone()
            .filter(|value| !value.trim().is_empty()),
        name: (!keyword.value.trim().is_empty()).then(|| keyword.value.trim().to_string()),
        keyword_value: keyword.value.trim().to_string(),
        value: result_value(element),
    })
}

pub(super) fn inline_result_from_object(
    object: &Object<ParsedAnnotation>,
) -> Option<SourceBlockResult> {
    let ObjectData::Macro { name, arguments } = &object.data else {
        return None;
    };
    if !name.eq_ignore_ascii_case("results") {
        return None;
    }
    let keyword_value = arguments.join(",");
    Some(SourceBlockResult {
        source: SourceBlockSource::from_annotation(&object.ann),
        kind: SourceBlockResultKind::InlineMacro,
        hash: None,
        name: None,
        value: keyword_value.clone(),
        keyword_value,
    })
}

fn result_value(element: &Element<ParsedAnnotation>) -> String {
    match &element.data {
        ElementData::FixedWidth(fixed) => fixed.value.trim_end().to_string(),
        ElementData::Block(block) => block.value.trim_end().to_string(),
        _ => strip_affiliated_result_prefix(element).trim().to_string(),
    }
}

fn strip_affiliated_result_prefix(element: &Element<ParsedAnnotation>) -> String {
    let mut value = element.ann.raw.as_str();
    for keyword in &element.affiliated_keywords {
        if keyword.key.eq_ignore_ascii_case("RESULTS") && value.starts_with(&keyword.ann.raw) {
            value = &value[keyword.ann.raw.len()..];
        }
    }
    value.to_string()
}
