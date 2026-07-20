//! Standard property, planning, headline, and provenance helpers for bridge index records.

use super::{
    OrgElementId, OrgElementProperties, OrgElementsIndexSummary, OrgElementsIndexSummaryValue,
    ParsedAnnotation, Section,
};
use crate::ast::elements_bridge_model::{
    OrgElementPropertyProvenance, OrgElementPropertyProvenanceMap,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct StandardProperties {
    pub(super) contents_begin: Option<usize>,
    pub(super) contents_end: Option<usize>,
    pub(super) post_blank: Option<usize>,
}

impl StandardProperties {
    pub(super) fn from_content_ann(ann: &ParsedAnnotation) -> Self {
        Self {
            contents_begin: Some(range_start(ann)),
            contents_end: Some(range_end(ann)),
            post_blank: None,
        }
    }
}

pub(super) fn properties_with_standard_properties(
    mut properties: OrgElementProperties,
    property_provenance: &mut OrgElementPropertyProvenanceMap,
    ann: &ParsedAnnotation,
    parent_id: Option<OrgElementId>,
    standard: StandardProperties,
) -> OrgElementProperties {
    let begin = range_start(ann);
    let end = range_end(ann);
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":begin",
        begin.into(),
        OrgElementPropertyProvenance::Standard,
    );
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":end",
        end.into(),
        OrgElementPropertyProvenance::Standard,
    );
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":post-affiliated",
        begin.into(),
        OrgElementPropertyProvenance::Standard,
    );
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":contents-begin",
        optional_usize(standard.contents_begin),
        OrgElementPropertyProvenance::Standard,
    );
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":contents-end",
        optional_usize(standard.contents_end),
        OrgElementPropertyProvenance::Standard,
    );
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":post-blank",
        optional_usize(standard.post_blank),
        OrgElementPropertyProvenance::Standard,
    );
    insert_property_with_provenance(
        &mut properties,
        property_provenance,
        ":parent",
        optional_usize(parent_id.map(OrgElementId::as_usize)),
        OrgElementPropertyProvenance::Standard,
    );
    properties
}

pub(super) fn range_start(ann: &ParsedAnnotation) -> usize {
    u32::from(ann.range.start()) as usize
}

pub(super) fn range_end(ann: &ParsedAnnotation) -> usize {
    u32::from(ann.range.end()) as usize
}

pub(super) fn has_planning(planning: &super::Planning) -> bool {
    planning.scheduled.is_some() || planning.deadline.is_some() || planning.closed.is_some()
}

pub(super) fn planning_summary(planning: &super::Planning) -> OrgElementsIndexSummary {
    summary([
        (
            "scheduled",
            optional_text(
                planning
                    .scheduled
                    .as_ref()
                    .map(|timestamp| timestamp.raw.as_str()),
            ),
        ),
        (
            "deadline",
            optional_text(
                planning
                    .deadline
                    .as_ref()
                    .map(|timestamp| timestamp.raw.as_str()),
            ),
        ),
        (
            "closed",
            optional_text(
                planning
                    .closed
                    .as_ref()
                    .map(|timestamp| timestamp.raw.as_str()),
            ),
        ),
    ])
}

pub(super) fn headline_properties(
    section: &Section<ParsedAnnotation>,
    summary: &OrgElementsIndexSummary,
) -> (OrgElementProperties, OrgElementPropertyProvenanceMap) {
    let mut properties = properties_from_summary(summary);
    let mut provenance = property_provenance_from_summary(summary);
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":raw-value",
        section.raw_title.trim_end().into(),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":title",
        section.raw_title.trim_end().into(),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":level",
        section.level.into(),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":true-level",
        section.level.into(),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":todo-keyword",
        optional_text(section.todo.as_ref().map(|todo| todo.name.as_str())),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":todo-type",
        optional_text(
            section
                .todo
                .as_ref()
                .map(|todo| todo_state_label(todo.state)),
        ),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":priority",
        optional_text(section.priority.raw_cookie()),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":tags",
        section.tags.clone().into(),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":scheduled",
        optional_text(
            section
                .planning
                .scheduled
                .as_ref()
                .map(|timestamp| timestamp.raw.as_str()),
        ),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":deadline",
        optional_text(
            section
                .planning
                .deadline
                .as_ref()
                .map(|timestamp| timestamp.raw.as_str()),
        ),
        OrgElementPropertyProvenance::Local,
    );
    insert_property_with_provenance(
        &mut properties,
        &mut provenance,
        ":closed",
        optional_text(
            section
                .planning
                .closed
                .as_ref()
                .map(|timestamp| timestamp.raw.as_str()),
        ),
        OrgElementPropertyProvenance::Local,
    );
    let local_property_keys = section
        .properties
        .iter()
        .flat_map(|property| {
            [
                org_property_key(&property.key),
                org_property_key(&property.key.to_ascii_uppercase()),
            ]
        })
        .collect::<std::collections::BTreeSet<_>>();
    for property in &section.effective_properties {
        let property_provenance = if local_property_keys.contains(&org_property_key(&property.key))
        {
            OrgElementPropertyProvenance::Local
        } else {
            OrgElementPropertyProvenance::Inherited
        };
        insert_property_with_provenance(
            &mut properties,
            &mut provenance,
            format!(":{}", property.key),
            property.value.clone().into(),
            property_provenance,
        );
        insert_property_with_provenance(
            &mut properties,
            &mut provenance,
            format!(":{}", property.key.to_ascii_uppercase()),
            property.value.clone().into(),
            property_provenance,
        );
    }
    (properties, provenance)
}

pub(super) fn insert_property_with_provenance(
    properties: &mut OrgElementProperties,
    provenance: &mut OrgElementPropertyProvenanceMap,
    key: impl AsRef<str>,
    value: OrgElementsIndexSummaryValue,
    property_provenance: OrgElementPropertyProvenance,
) {
    let key = org_property_key(key.as_ref());
    properties.insert(key.clone(), value);
    provenance.insert(key, property_provenance);
}

pub(super) fn org_property_key(key: &str) -> String {
    if key.starts_with(':') {
        key.to_string()
    } else {
        format!(":{key}")
    }
}
use super::elements_bridge_index_summary::{
    optional_text, optional_usize, properties_from_summary, property_provenance_from_summary,
    summary, todo_state_label,
};
