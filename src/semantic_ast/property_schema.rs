//! Property schema contract validation over ordinary Org property drawers.

use super::{
    PROPERTY_SCHEMA_PROPERTY, ParsedAnnotation, Property, PropertySchemaApplication,
    PropertySchemaContract, PropertySchemaFinding, PropertySchemaFindingKind,
    PropertySchemaReference, PropertySchemaReferenceKind, PropertySchemaRegistry,
    PropertySchemaScope, PropertySchemaValueRule, Section, SectionIndexSource,
};

pub(super) fn property_schema_applications(
    document_properties: &[Property<ParsedAnnotation>],
    sections: &[Section<ParsedAnnotation>],
    registry: &PropertySchemaRegistry,
) -> Vec<PropertySchemaApplication> {
    let mut applications = Vec::new();
    push_property_schema_application(
        &mut applications,
        document_properties,
        PropertySchemaScope::Document,
        registry,
    );
    for section in sections {
        collect_section_schema_applications(&mut applications, section, &mut Vec::new(), registry);
    }
    applications
}

fn collect_section_schema_applications(
    applications: &mut Vec<PropertySchemaApplication>,
    section: &Section<ParsedAnnotation>,
    outline_path: &mut Vec<String>,
    registry: &PropertySchemaRegistry,
) {
    outline_path.push(section.raw_title.clone());
    push_property_schema_application(
        applications,
        &section.properties,
        PropertySchemaScope::Section {
            outline_path: outline_path.clone(),
            level: section.level,
            title: section.raw_title.clone(),
        },
        registry,
    );
    for child in &section.subsections {
        collect_section_schema_applications(applications, child, outline_path, registry);
    }
    outline_path.pop();
}

fn push_property_schema_application(
    applications: &mut Vec<PropertySchemaApplication>,
    properties: &[Property<ParsedAnnotation>],
    scope: PropertySchemaScope,
    registry: &PropertySchemaRegistry,
) {
    let Some(schema_property) = schema_property(properties) else {
        return;
    };
    let source = SectionIndexSource::from_annotation(&schema_property.ann);
    let reference = property_schema_reference(schema_property.value.as_str());
    let mut findings = Vec::new();
    let contract = if reference.kind == PropertySchemaReferenceKind::Empty {
        findings.push(schema_finding(
            source.clone(),
            PropertySchemaFindingKind::EmptyReference,
            Some(PROPERTY_SCHEMA_PROPERTY),
            Some(schema_property.value.clone()),
            Vec::new(),
            "PROPERTY_SCHEMA is empty; load or choose a schema contract id",
        ));
        None
    } else {
        registry.resolve(&reference)
    };

    let contract_id = contract.map(|contract| contract.id.clone());
    if let Some(contract) = contract {
        findings.extend(validate_properties_against_contract(
            properties, contract, &source,
        ));
    } else if reference.kind != PropertySchemaReferenceKind::Empty {
        findings.push(schema_finding(
            source.clone(),
            PropertySchemaFindingKind::UnresolvedReference,
            Some(PROPERTY_SCHEMA_PROPERTY),
            Some(reference.raw.clone()),
            Vec::new(),
            format!(
                "PROPERTY_SCHEMA `{}` was not found in the loaded schema registry",
                reference.raw
            ),
        ));
    }

    applications.push(PropertySchemaApplication {
        source,
        scope,
        reference,
        contract_id,
        findings,
    });
}

fn validate_properties_against_contract(
    properties: &[Property<ParsedAnnotation>],
    contract: &PropertySchemaContract,
    schema_source: &SectionIndexSource,
) -> Vec<PropertySchemaFinding> {
    let mut findings = Vec::new();
    for field in &contract.fields {
        let Some(property) = property_by_key(properties, field.key.as_str()) else {
            if field.required {
                findings.push(schema_finding(
                    schema_source.clone(),
                    PropertySchemaFindingKind::MissingRequiredProperty,
                    Some(field.key.clone()),
                    None,
                    Vec::new(),
                    format!("property schema `{}` requires `{}`", contract.id, field.key),
                ));
            }
            continue;
        };
        findings.extend(validate_property_value(property, &field.value_rule));
    }

    if !contract.allow_unknown_properties {
        for property in properties {
            if property.key.eq_ignore_ascii_case(PROPERTY_SCHEMA_PROPERTY)
                || property.key.to_ascii_uppercase().ends_with("_ALL")
                || contract.field_for(&property.key).is_some()
            {
                continue;
            }
            findings.push(schema_finding(
                SectionIndexSource::from_annotation(&property.ann),
                PropertySchemaFindingKind::UnknownProperty,
                Some(property.key.clone()),
                Some(property.value.clone()),
                contract
                    .fields
                    .iter()
                    .map(|field| field.key.clone())
                    .collect(),
                format!(
                    "property `{}` is not declared by schema `{}`",
                    property.key, contract.id
                ),
            ));
        }
    }

    findings
}

fn validate_property_value(
    property: &Property<ParsedAnnotation>,
    value_rule: &PropertySchemaValueRule,
) -> Vec<PropertySchemaFinding> {
    match value_rule {
        PropertySchemaValueRule::Any => Vec::new(),
        PropertySchemaValueRule::NonEmpty => {
            if property.value.trim().is_empty() {
                vec![schema_finding(
                    SectionIndexSource::from_annotation(&property.ann),
                    PropertySchemaFindingKind::EmptyValue,
                    Some(property.key.clone()),
                    Some(property.value.clone()),
                    Vec::new(),
                    format!("property `{}` must not be empty", property.key),
                )]
            } else {
                Vec::new()
            }
        }
        PropertySchemaValueRule::OneOf(values) => {
            if values.iter().any(|value| value == &property.value) {
                Vec::new()
            } else {
                vec![schema_finding(
                    SectionIndexSource::from_annotation(&property.ann),
                    PropertySchemaFindingKind::DisallowedValue,
                    Some(property.key.clone()),
                    Some(property.value.clone()),
                    values.clone(),
                    format!(
                        "property `{}` value `{}` is not allowed by schema: {}",
                        property.key,
                        property.value,
                        values.join(", ")
                    ),
                )]
            }
        }
    }
}

fn schema_property(
    properties: &[Property<ParsedAnnotation>],
) -> Option<&Property<ParsedAnnotation>> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(PROPERTY_SCHEMA_PROPERTY))
}

fn property_by_key<'a>(
    properties: &'a [Property<ParsedAnnotation>],
    key: &str,
) -> Option<&'a Property<ParsedAnnotation>> {
    properties
        .iter()
        .rev()
        .find(|property| property.key.eq_ignore_ascii_case(key))
}

fn property_schema_reference(value: &str) -> PropertySchemaReference {
    let raw = value.trim().to_string();
    if raw.is_empty() {
        return PropertySchemaReference {
            raw,
            normalized: String::new(),
            kind: PropertySchemaReferenceKind::Empty,
        };
    }
    if let Some(target) = org_file_link_target(raw.as_str()) {
        return PropertySchemaReference {
            raw,
            normalized: target,
            kind: PropertySchemaReferenceKind::OrgFileLink,
        };
    }
    if let Some(argument) = macro_reference_argument(raw.as_str()) {
        return PropertySchemaReference {
            raw,
            normalized: argument,
            kind: PropertySchemaReferenceKind::Macro,
        };
    }
    let kind = if is_file_reference(raw.as_str()) {
        PropertySchemaReferenceKind::File
    } else {
        PropertySchemaReferenceKind::ContractId
    };
    PropertySchemaReference {
        normalized: raw.clone(),
        raw,
        kind,
    }
}

fn org_file_link_target(value: &str) -> Option<String> {
    let inner = value.strip_prefix("[[")?.strip_suffix("]]")?;
    let target = inner.split("][").next().unwrap_or(inner).trim();
    target
        .starts_with("file:")
        .then(|| target.to_string())
        .filter(|target| !target.is_empty())
}

fn macro_reference_argument(value: &str) -> Option<String> {
    let inner = value.strip_prefix("{{{")?.strip_suffix("}}}")?.trim();
    let start = inner.find('(')?;
    let end = inner.rfind(')')?;
    (end > start + 1)
        .then(|| inner[start + 1..end].trim().to_string())
        .filter(|argument| !argument.is_empty())
}

fn is_file_reference(value: &str) -> bool {
    let without_fragment = value
        .split_once('#')
        .map_or(value, |(path, _fragment)| path);
    let path = without_fragment
        .split_once('?')
        .map_or(without_fragment, |(path, _query)| path);
    path.starts_with("file:")
        || path.starts_with("./")
        || path.starts_with("../")
        || path.ends_with(".json")
        || path.ends_with(".toml")
        || path.ends_with(".yaml")
        || path.ends_with(".yml")
        || path.contains('/')
}

fn schema_finding(
    source: SectionIndexSource,
    kind: PropertySchemaFindingKind,
    property: Option<impl Into<String>>,
    actual: Option<String>,
    expected: Vec<String>,
    message: impl Into<String>,
) -> PropertySchemaFinding {
    PropertySchemaFinding {
        source,
        kind,
        property: property.map(Into::into),
        actual,
        expected,
        message: message.into(),
    }
}
