//! Org-native SDD projections.

use super::{
    Document, ParsedAnnotation, SddKind, SddNodeRecord, SddParentRef, SddStatus, Section,
    SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Projects Org-native SDD nodes from headings and property drawers.
    ///
    /// This is document-local and read-only. It does not discover other files,
    /// write archives, or mutate SDD tasks.
    pub fn sdd_node_records(&self) -> Vec<SddNodeRecord> {
        let mut records = Vec::new();
        for section in &self.sections {
            collect_sdd_nodes(section, Vec::new(), &mut records);
        }
        records.sort_by_key(|record| record.source.range_start);
        records
    }

    /// Projects a compact document-local SDD status snapshot.
    pub fn sdd_status(&self) -> SddStatus {
        SddStatus {
            records: self.sdd_node_records(),
        }
    }
}

fn collect_sdd_nodes(
    section: &Section<ParsedAnnotation>,
    parent_outline_path: Vec<String>,
    records: &mut Vec<SddNodeRecord>,
) {
    let title = section.raw_title.trim_end().to_string();
    let mut outline_path = parent_outline_path;
    outline_path.push(title.clone());

    if is_sdd_section(section) {
        records.push(sdd_node_record(section, outline_path.clone(), title));
    }

    for child in &section.subsections {
        collect_sdd_nodes(child, outline_path.clone(), records);
    }
}

fn sdd_node_record(
    section: &Section<ParsedAnnotation>,
    outline_path: Vec<String>,
    title: String,
) -> SddNodeRecord {
    SddNodeRecord {
        source: SectionIndexSource::from_annotation(&section.ann),
        outline_path,
        level: section.level,
        title,
        kind: local_property(section, "SDD_KIND").map_or_else(
            || SddKind::Unknown(String::new()),
            |value| SddKind::parse(value),
        ),
        id: local_property(section, "ID").map(str::to_string),
        parent: local_property(section, "SDD_PARENT").and_then(SddParentRef::parse),
        capability: local_property(section, "SDD_CAPABILITY").map(str::to_string),
        slug: local_property(section, "SDD_SLUG").map(str::to_string),
        status: local_property(section, "SDD_STATUS").map(str::to_string),
        todo: section.todo.clone(),
        tags: section.tags.clone(),
    }
}

pub(crate) fn is_sdd_section(section: &Section<ParsedAnnotation>) -> bool {
    has_local_tag(section, "sdd") || local_property(section, "SDD_KIND").is_some()
}

pub(crate) fn local_property<'a>(
    section: &'a Section<ParsedAnnotation>,
    key: &str,
) -> Option<&'a str> {
    section
        .properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| property.value.trim())
}

fn has_local_tag(section: &Section<ParsedAnnotation>, tag: &str) -> bool {
    section.tags.iter().any(|value| value == tag)
}
