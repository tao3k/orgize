//! Dynamic block registry projection over parsed Org blocks.

use super::{
    BlockHeaderArg, BlockKind, Document, DynamicBlockContentState, DynamicBlockParameter,
    DynamicBlockRecord, DynamicBlockWriterKind, Element, ElementData, ParsedAnnotation, Section,
    SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Projects native Org dynamic blocks without executing their writer functions.
    pub fn dynamic_block_records(&self) -> Vec<DynamicBlockRecord> {
        let mut records = Vec::new();
        collect_dynamic_blocks_in_elements(&self.children, &mut records);
        for section in &self.sections {
            collect_dynamic_blocks_in_section(section, &mut records);
        }
        records.sort_by_key(|record| record.source.range_start);
        records
    }
}

fn collect_dynamic_blocks_in_section(
    section: &Section<ParsedAnnotation>,
    records: &mut Vec<DynamicBlockRecord>,
) {
    collect_dynamic_blocks_in_elements(&section.children, records);
    for child in &section.subsections {
        collect_dynamic_blocks_in_section(child, records);
    }
}

fn collect_dynamic_blocks_in_elements(
    elements: &[Element<ParsedAnnotation>],
    records: &mut Vec<DynamicBlockRecord>,
) {
    for element in elements {
        if let Some(record) = dynamic_block_record(element) {
            records.push(record);
        }
        match &element.data {
            ElementData::Drawer(drawer) => {
                collect_dynamic_blocks_in_elements(&drawer.children, records)
            }
            ElementData::List(list) => {
                for item in &list.items {
                    collect_dynamic_blocks_in_elements(&item.children, records);
                }
            }
            ElementData::Block(block) => {
                collect_dynamic_blocks_in_elements(&block.children, records)
            }
            ElementData::FootnoteDef(footnote) => {
                collect_dynamic_blocks_in_elements(&footnote.children, records);
            }
            ElementData::Inlinetask(task) => {
                collect_dynamic_blocks_in_elements(&task.children, records);
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

fn dynamic_block_record(element: &Element<ParsedAnnotation>) -> Option<DynamicBlockRecord> {
    let ElementData::Block(block) = &element.data else {
        return None;
    };
    if block.kind != BlockKind::Dynamic {
        return None;
    }
    let name = block.name.as_ref()?;
    let (content_state, content_line_count) = dynamic_block_content(&element.ann.raw);
    Some(DynamicBlockRecord {
        source: SectionIndexSource::from_annotation(&element.ann),
        writer: writer_kind(name),
        name: name.clone(),
        parameters: block
            .header_args
            .iter()
            .cloned()
            .map(dynamic_block_parameter)
            .collect(),
        content_state,
        content_line_count,
    })
}

fn dynamic_block_parameter(parameter: BlockHeaderArg) -> DynamicBlockParameter {
    DynamicBlockParameter {
        key: parameter.key,
        value: parameter.value,
        raw: parameter.raw,
    }
}

fn writer_kind(name: &str) -> DynamicBlockWriterKind {
    if name.eq_ignore_ascii_case("clocktable") {
        DynamicBlockWriterKind::ClockTable
    } else if name.eq_ignore_ascii_case("columnview") {
        DynamicBlockWriterKind::ColumnView
    } else {
        DynamicBlockWriterKind::Unknown
    }
}

fn dynamic_block_content(raw: &str) -> (DynamicBlockContentState, usize) {
    // The Scheme/Rowan block range already identifies the closing line.
    let lines = raw.lines().collect::<Vec<_>>();
    let body = lines
        .get(1..lines.len().saturating_sub(1))
        .unwrap_or_default();
    let (has_nonblank_content, content_line_count) = body
        .iter()
        .fold((false, 0usize), |(has_nonblank, count), line| {
            (has_nonblank || !line.trim().is_empty(), count + 1)
        });

    (
        if has_nonblank_content {
            DynamicBlockContentState::ExistingOutput
        } else {
            DynamicBlockContentState::Empty
        },
        content_line_count,
    )
}
