//! Property profile projection over native Org property descriptors.

use super::property_plan::{FIXED, PropertyNativePlan};
use super::{
    Document, ParsedAnnotation, Property, PropertyAllowedValueRecord, PropertyAllowedValueScope,
    PropertyInheritancePolicy, PropertyProfile, PropertySchemaRegistry, Section,
    SectionIndexSource,
};

impl Document<ParsedAnnotation> {
    /// Projects inheritance metadata and `PROPERTY_ALL` allowed-value descriptors.
    pub fn property_profile(&self) -> PropertyProfile {
        self.property_profile_with_schema_registry(&PropertySchemaRegistry::default())
    }

    /// Projects property metadata and validates loaded `PROPERTY_SCHEMA` contracts.
    pub fn property_profile_with_schema_registry(
        &self,
        registry: &PropertySchemaRegistry,
    ) -> PropertyProfile {
        self.property_profile_with_native_plan(registry).0
    }

    pub(crate) fn property_profile_with_native_plan(
        &self,
        registry: &PropertySchemaRegistry,
    ) -> (PropertyProfile, PropertyNativePlan) {
        let plan = PropertyNativePlan::new(self);
        let mut inherited_keys = Vec::new();
        let mut allowed_values = fixed_global_allowed_values(&plan);
        for property in &self.properties {
            push_inherited_key(&mut inherited_keys, &property.key);
            push_allowed_value_record(
                &mut allowed_values,
                property,
                PropertyAllowedValueScope::Document,
                &plan,
            );
        }
        for section in &self.sections {
            collect_section_property_profile(
                section,
                &mut Vec::new(),
                &mut inherited_keys,
                &mut allowed_values,
                &plan,
            );
        }

        (
            PropertyProfile {
                inheritance: PropertyInheritancePolicy::All,
                inherited_keys,
                allowed_values,
                schema_applications: super::property_schema::property_schema_applications(
                    &self.properties,
                    &self.sections,
                    registry,
                ),
            },
            plan,
        )
    }
}

pub(crate) fn property_allowed_values(
    properties: &[Property<ParsedAnnotation>],
    profile: &PropertyProfile,
    property: &Property<ParsedAnnotation>,
    plan: &PropertyNativePlan,
) -> Option<Vec<String>> {
    let descriptor_key = &plan.property(property).descriptor_key;
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(descriptor_key))
        .map(|property| plan.property(property).tokens.clone())
        .or_else(|| fixed_global_allowed_values_for(profile, descriptor_key))
}

fn collect_section_property_profile(
    section: &Section<ParsedAnnotation>,
    outline_path: &mut Vec<String>,
    inherited_keys: &mut Vec<String>,
    allowed_values: &mut Vec<PropertyAllowedValueRecord>,
    plan: &PropertyNativePlan,
) {
    outline_path.push(section.raw_title.clone());
    for property in &section.properties {
        push_inherited_key(inherited_keys, &property.key);
        push_allowed_value_record(
            allowed_values,
            property,
            PropertyAllowedValueScope::Section {
                outline_path: outline_path.clone(),
                level: section.level,
                title: section.raw_title.clone(),
            },
            plan,
        );
    }
    for child in &section.subsections {
        collect_section_property_profile(child, outline_path, inherited_keys, allowed_values, plan);
    }
    outline_path.pop();
}

fn push_allowed_value_record(
    allowed_values: &mut Vec<PropertyAllowedValueRecord>,
    property: &Property<ParsedAnnotation>,
    scope: PropertyAllowedValueScope,
    plan: &PropertyNativePlan,
) {
    let facts = plan.property(property);
    if let Some(property_name) = &facts.descriptor_name {
        allowed_values.push(PropertyAllowedValueRecord {
            source: Some(SectionIndexSource::from_annotation(&property.ann)),
            scope,
            property: property_name.clone(),
            descriptor_key: property.key.clone(),
            values: facts.tokens.clone(),
        });
    }
}

fn push_inherited_key(keys: &mut Vec<String>, key: &str) {
    if !keys
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(key))
    {
        keys.push(key.to_string());
    }
}

fn fixed_global_allowed_values(plan: &PropertyNativePlan) -> Vec<PropertyAllowedValueRecord> {
    FIXED
        .into_iter()
        .map(|(descriptor_key, value)| PropertyAllowedValueRecord {
            source: None,
            scope: PropertyAllowedValueScope::FixedGlobal,
            property: plan
                .fact(descriptor_key, value)
                .descriptor_name
                .clone()
                .expect("fixed descriptor key should end in _ALL"),
            descriptor_key: descriptor_key.to_string(),
            values: plan.fact(descriptor_key, value).tokens.clone(),
        })
        .collect()
}

fn fixed_global_allowed_values_for(
    profile: &PropertyProfile,
    descriptor_key: &str,
) -> Option<Vec<String>> {
    profile
        .allowed_values
        .iter()
        .find(|record| {
            matches!(record.scope, PropertyAllowedValueScope::FixedGlobal)
                && record.descriptor_key.eq_ignore_ascii_case(descriptor_key)
        })
        .map(|record| record.values.clone())
}
