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
    super::org_values::rows("column-values", &[value])
        .into_iter()
        .map(|row| {
            let [property, title, title_present, width, operator, format, raw]: [String; 7] =
                row.try_into().expect("native column arity");
            let title = match title_present.as_str() {
                "true" => Some(title),
                "false" => {
                    assert!(title.is_empty());
                    None
                }
                _ => unreachable!("native column title presence"),
            };
            ColumnViewColumn {
                property,
                title,
                width: (!width.is_empty()).then(|| width.parse().expect("native column width")),
                summary_operator: (!operator.is_empty()).then_some(operator),
                summary_format: (!format.is_empty()).then_some(format),
                raw,
            }
        })
        .collect()
}
