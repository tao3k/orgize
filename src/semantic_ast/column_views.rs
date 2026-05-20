//! Column View side-table projection for document and subtree metadata.

use super::{
    ColumnViewColumn, ColumnViewRecord, ColumnViewScope, ColumnViewSource, Document, ElementData,
    Keyword, ParsedAnnotation, Property, Section,
};

impl Document<ParsedAnnotation> {
    /// Projects `#+COLUMNS:` keywords and `COLUMNS` properties into typed records.
    ///
    /// Org column view format is kept declarative here. This projection parses
    /// column declarations without computing property inheritance or summaries.
    pub fn column_view_records(&self) -> Vec<ColumnViewRecord> {
        let mut records = Vec::new();
        records.extend(self.metadata.iter().filter_map(document_columns_keyword));
        records.extend(self.children.iter().filter_map(|element| {
            let ElementData::Keyword(keyword) = &element.data else {
                return None;
            };
            document_columns_keyword(keyword)
        }));
        records.extend(self.properties.iter().filter_map(document_columns_property));

        let mut outline_path = Vec::new();
        for section in &self.sections {
            collect_section_column_views(section, &mut outline_path, &mut records);
        }
        records
    }
}

fn document_columns_keyword(keyword: &Keyword<ParsedAnnotation>) -> Option<ColumnViewRecord> {
    keyword
        .key
        .eq_ignore_ascii_case("COLUMNS")
        .then(|| ColumnViewRecord {
            source: ColumnViewSource::from_annotation(&keyword.ann),
            scope: ColumnViewScope::DocumentKeyword,
            raw: keyword.value.trim().to_string(),
            columns: column_view_columns(&keyword.value),
        })
}

fn document_columns_property(property: &Property<ParsedAnnotation>) -> Option<ColumnViewRecord> {
    property
        .key
        .eq_ignore_ascii_case("COLUMNS")
        .then(|| ColumnViewRecord {
            source: ColumnViewSource::from_annotation(&property.ann),
            scope: ColumnViewScope::DocumentProperty,
            raw: property.value.trim().to_string(),
            columns: column_view_columns(&property.value),
        })
}

fn collect_section_column_views(
    section: &Section<ParsedAnnotation>,
    outline_path: &mut Vec<String>,
    records: &mut Vec<ColumnViewRecord>,
) {
    outline_path.push(section.raw_title.trim().to_string());
    for property in &section.properties {
        if property.key.eq_ignore_ascii_case("COLUMNS") {
            push_section_column_property(section, outline_path, property, records);
        }
    }
    for element in &section.children {
        let ElementData::PropertyDrawer(properties) = &element.data else {
            continue;
        };
        for property in properties {
            if property.key.eq_ignore_ascii_case("COLUMNS")
                && !records.iter().any(|record| {
                    record.source.range_start == u32::from(property.ann.range.start())
                        && record.source.range_end == u32::from(property.ann.range.end())
                })
            {
                push_section_column_property(section, outline_path, property, records);
            }
        }
    }
    for subsection in &section.subsections {
        collect_section_column_views(subsection, outline_path, records);
    }
    outline_path.pop();
}

fn push_section_column_property(
    section: &Section<ParsedAnnotation>,
    outline_path: &[String],
    property: &Property<ParsedAnnotation>,
    records: &mut Vec<ColumnViewRecord>,
) {
    records.push(ColumnViewRecord {
        source: ColumnViewSource::from_annotation(&property.ann),
        scope: ColumnViewScope::SectionProperty {
            level: section.level,
            title: section.raw_title.trim().to_string(),
            outline_path: outline_path.to_vec(),
        },
        raw: property.value.trim().to_string(),
        columns: column_view_columns(&property.value),
    });
}

fn column_view_columns(value: &str) -> Vec<ColumnViewColumn> {
    value
        .split_whitespace()
        .filter_map(column_view_column)
        .collect()
}

fn column_view_column(raw: &str) -> Option<ColumnViewColumn> {
    let raw = raw.trim();
    let rest = raw.strip_prefix('%')?;
    let (width, rest) = column_view_width(rest);
    let property_end = rest
        .char_indices()
        .find(|(_, ch)| !is_column_property_char(*ch))
        .map(|(index, _)| index)
        .unwrap_or(rest.len());
    let property = rest[..property_end].trim();
    if property.is_empty() {
        return None;
    }

    let mut title = None;
    let mut summary_operator = None;
    let mut summary_format = None;
    let mut tail = &rest[property_end..];

    if let Some(after_open) = tail.strip_prefix('(')
        && let Some(close) = after_open.find(')')
    {
        title = Some(after_open[..close].to_string());
        tail = &after_open[close + 1..];
    }

    if let Some(after_open) = tail.strip_prefix('{')
        && let Some(close) = after_open.rfind('}')
    {
        let summary = &after_open[..close];
        let (operator, format) = summary.split_once(';').unwrap_or((summary, ""));
        summary_operator = (!operator.trim().is_empty()).then(|| operator.trim().to_string());
        summary_format = (!format.trim().is_empty()).then(|| format.trim().to_string());
    }

    Some(ColumnViewColumn {
        property: property.to_ascii_uppercase(),
        title,
        width,
        summary_operator,
        summary_format,
        raw: raw.to_string(),
    })
}

fn column_view_width(value: &str) -> (Option<usize>, &str) {
    let width_end = value
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_digit())
        .map(|(index, _)| index)
        .unwrap_or(value.len());
    if width_end == 0 {
        (None, value)
    } else {
        (value[..width_end].parse().ok(), &value[width_end..])
    }
}

fn is_column_property_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}
