//! Flat org-element-map-style index for the Org elements bridge.

use super::{
    Document, Element, OrgElementId, OrgElementProperties, OrgElementsAffiliatedProperties,
    OrgElementsIndexCategory, OrgElementsIndexRecord, OrgElementsIndexSummary, ParsedAnnotation,
};
use crate::ast::elements_bridge_model::{
    OrgElementPropertyProvenance, OrgElementPropertyProvenanceMap,
};

use super::elements_bridge_index_properties::StandardProperties;
use super::elements_bridge_index_summary::summary;
pub(super) fn index_records(
    document: &Document<ParsedAnnotation>,
) -> Vec<OrgElementsIndexRecord<ParsedAnnotation>> {
    let mut index = ElementIndex::default();
    let root_id = index.push(ElementIndexRecordSpec::new(
        None,
        OrgElementsIndexCategory::Document,
        "org-data",
        &document.ann,
        &[],
        "root",
        summary([
            ("sections", document.sections.len().into()),
            ("elements", document.children.len().into()),
            ("metadata", document.metadata.len().into()),
        ]),
    ));
    for keyword in &document.metadata {
        index.push_keyword(keyword, &[], "metadata", root_id);
    }
    index.collect_property_drawer(&document.properties, &[], "document", root_id);
    for target in &document.targets {
        index.push(ElementIndexRecordSpec::new(
            Some(root_id),
            OrgElementsIndexCategory::TargetDefinition,
            "target-definition",
            &target.ann,
            &[],
            "sideTable",
            summary([
                ("key", target.key.clone().into()),
                ("value", target.value.clone().into()),
                ("targetKind", target_kind(target.kind).into()),
            ]),
        ));
        index.collect_objects(&target.alias, &[], "targetAlias", root_id);
    }
    for footnote in &document.footnotes {
        index.push(ElementIndexRecordSpec::new(
            Some(root_id),
            OrgElementsIndexCategory::FootnoteEntry,
            "footnote-entry",
            &footnote.ann,
            &[],
            "sideTable",
            summary([("label", footnote.label.clone().into())]),
        ));
    }
    index.collect_elements(&document.children, &[], "document", root_id);
    index.collect_sections(&document.sections, Vec::new(), root_id);
    index.records
}

#[derive(Default)]
pub(super) struct ElementIndex {
    pub(super) next_ordinal: usize,
    pub(super) records: Vec<OrgElementsIndexRecord<ParsedAnnotation>>,
}

pub(super) struct ElementIndexRecordSpec<'a> {
    pub(super) parent_id: Option<OrgElementId>,
    pub(super) category: OrgElementsIndexCategory,
    pub(super) kind: &'a str,
    pub(super) ann: &'a ParsedAnnotation,
    pub(super) outline_path: &'a [String],
    pub(super) context: &'a str,
    pub(super) properties: OrgElementProperties,
    pub(super) property_provenance: OrgElementPropertyProvenanceMap,
    pub(super) summary: OrgElementsIndexSummary,
    pub(super) standard: StandardProperties,
}

impl<'a> ElementIndexRecordSpec<'a> {
    pub(super) fn new(
        parent_id: Option<OrgElementId>,
        category: OrgElementsIndexCategory,
        kind: &'a str,
        ann: &'a ParsedAnnotation,
        outline_path: &'a [String],
        context: &'a str,
        summary: OrgElementsIndexSummary,
    ) -> Self {
        Self {
            parent_id,
            category,
            kind,
            ann,
            outline_path,
            context,
            properties: properties_from_summary(&summary),
            property_provenance: property_provenance_from_summary(&summary),
            summary,
            standard: StandardProperties::default(),
        }
    }

    pub(super) fn with_properties(mut self, properties: OrgElementProperties) -> Self {
        self.property_provenance =
            property_provenance_from_properties(&properties, OrgElementPropertyProvenance::Summary);
        self.properties = properties;
        self
    }

    pub(super) fn with_property_provenance(
        mut self,
        property_provenance: OrgElementPropertyProvenanceMap,
    ) -> Self {
        self.property_provenance = property_provenance;
        self
    }

    pub(super) fn with_standard_properties(mut self, standard: StandardProperties) -> Self {
        self.standard = standard;
        self
    }
}

impl ElementIndex {
    pub(super) fn push(&mut self, input: ElementIndexRecordSpec<'_>) -> OrgElementId {
        self.next_ordinal += 1;
        let id = OrgElementId::new(self.next_ordinal);
        let mut property_provenance = input.property_provenance;
        let properties = properties_with_standard_properties(
            input.properties,
            &mut property_provenance,
            input.ann,
            input.parent_id,
            input.standard,
        );
        self.records.push(OrgElementsIndexRecord {
            id,
            parent_id: input.parent_id,
            child_ids: Vec::new(),
            ann: input.ann.clone(),
            ordinal: self.next_ordinal,
            category: input.category,
            kind: input.kind.into(),
            affiliated: OrgElementsAffiliatedProperties::default(),
            outline_path: input.outline_path.to_vec(),
            context: input.context.to_string(),
            properties,
            property_provenance,
            summary: input.summary,
        });
        if let Some(parent_id) = input.parent_id
            && let Some(parent) = self
                .records
                .iter_mut()
                .find(|record| record.id == parent_id)
        {
            parent.child_ids.push(id);
        }
        id
    }

    pub(super) fn push_element(
        &mut self,
        parent_id: OrgElementId,
        element: &Element<ParsedAnnotation>,
        outline_path: &[String],
        context: impl Into<String>,
        summary: OrgElementsIndexSummary,
    ) -> OrgElementId {
        self.next_ordinal += 1;
        let id = OrgElementId::new(self.next_ordinal);
        let mut property_provenance = property_provenance_from_summary(&summary);
        let properties = properties_with_standard_properties(
            properties_from_summary(&summary),
            &mut property_provenance,
            &element.ann,
            Some(parent_id),
            StandardProperties::default(),
        );
        self.records.push(OrgElementsIndexRecord {
            id,
            parent_id: Some(parent_id),
            child_ids: Vec::new(),
            ann: element.ann.clone(),
            ordinal: self.next_ordinal,
            category: OrgElementsIndexCategory::Element,
            kind: element_kind(element).into(),
            affiliated: element_affiliated_properties(element),
            outline_path: outline_path.to_vec(),
            context: context.into(),
            properties,
            property_provenance,
            summary,
        });
        if let Some(parent) = self
            .records
            .iter_mut()
            .find(|record| record.id == parent_id)
        {
            parent.child_ids.push(id);
        }
        id
    }
}
use super::elements_bridge_index_properties::properties_with_standard_properties;
use super::elements_bridge_index_summary::{
    element_affiliated_properties, element_kind, properties_from_summary,
    property_provenance_from_properties, property_provenance_from_summary, target_kind,
};
