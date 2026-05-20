//! Property profile projection over native Org property descriptors.

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
        let mut inherited_keys = Vec::new();
        let mut allowed_values = fixed_global_allowed_values();
        for property in &self.properties {
            push_inherited_key(&mut inherited_keys, &property.key);
            push_allowed_value_record(
                &mut allowed_values,
                property,
                PropertyAllowedValueScope::Document,
            );
        }
        for section in &self.sections {
            collect_section_property_profile(
                section,
                &mut Vec::new(),
                &mut inherited_keys,
                &mut allowed_values,
            );
        }

        PropertyProfile {
            inheritance: PropertyInheritancePolicy::All,
            inherited_keys,
            allowed_values,
            schema_applications: super::property_schema::property_schema_applications(
                &self.properties,
                &self.sections,
                registry,
            ),
        }
    }
}

pub(crate) fn property_allowed_values(
    properties: &[Property<ParsedAnnotation>],
    profile: &PropertyProfile,
    key: &str,
) -> Option<Vec<String>> {
    let descriptor_key = allowed_value_descriptor_key(key);
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(&descriptor_key))
        .map(|property| allowed_value_tokens(&property.value))
        .or_else(|| fixed_global_allowed_values_for(profile, &descriptor_key))
}

pub(crate) fn is_allowed_value_descriptor(key: &str) -> bool {
    descriptor_property_name(key).is_some()
}

fn collect_section_property_profile(
    section: &Section<ParsedAnnotation>,
    outline_path: &mut Vec<String>,
    inherited_keys: &mut Vec<String>,
    allowed_values: &mut Vec<PropertyAllowedValueRecord>,
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
        );
    }
    for child in &section.subsections {
        collect_section_property_profile(child, outline_path, inherited_keys, allowed_values);
    }
    outline_path.pop();
}

fn push_allowed_value_record(
    allowed_values: &mut Vec<PropertyAllowedValueRecord>,
    property: &Property<ParsedAnnotation>,
    scope: PropertyAllowedValueScope,
) {
    if let Some(property_name) = descriptor_property_name(&property.key) {
        allowed_values.push(PropertyAllowedValueRecord {
            source: Some(SectionIndexSource::from_annotation(&property.ann)),
            scope,
            property: property_name,
            descriptor_key: property.key.clone(),
            values: allowed_value_tokens(&property.value),
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

fn fixed_global_allowed_values() -> Vec<PropertyAllowedValueRecord> {
    [
        ("VISIBILITY_ALL", "folded children content all"),
        ("CLOCK_MODELINE_TOTAL_ALL", "current today repeat all auto"),
    ]
    .into_iter()
    .map(|(descriptor_key, value)| PropertyAllowedValueRecord {
        source: None,
        scope: PropertyAllowedValueScope::FixedGlobal,
        property: descriptor_property_name(descriptor_key)
            .expect("fixed descriptor key should end in _ALL"),
        descriptor_key: descriptor_key.to_string(),
        values: allowed_value_tokens(value),
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

fn allowed_value_descriptor_key(key: &str) -> String {
    format!("{}_ALL", key.trim_end_matches('+'))
}

fn descriptor_property_name(key: &str) -> Option<String> {
    let trimmed = key.trim_end_matches('+');
    if !trimmed.to_ascii_uppercase().ends_with("_ALL") {
        return None;
    }
    let base = &trimmed[..trimmed.len() - "_ALL".len()];
    (!base.is_empty()).then(|| base.to_string())
}

fn allowed_value_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while let Some(start) = next_token_start(value, cursor) {
        let (token, next) = allowed_value_token(value, start);
        tokens.push(token);
        cursor = next;
    }
    tokens
}

fn next_token_start(value: &str, cursor: usize) -> Option<usize> {
    value[cursor..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| cursor + offset)
}

fn allowed_value_token(value: &str, start: usize) -> (String, usize) {
    let mut cursor = start;
    let mut parsed = String::new();
    let mut quote = None;
    let mut escaped = false;
    while cursor < value.len() {
        let ch = value[cursor..].chars().next().unwrap();
        cursor += ch.len_utf8();
        if escaped {
            parsed.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '"' | '\'') {
            quote = Some(ch);
        } else if quote.is_none() && ch.is_whitespace() {
            break;
        } else {
            parsed.push(ch);
        }
    }
    if escaped {
        parsed.push('\\');
    }
    (parsed, cursor)
}
