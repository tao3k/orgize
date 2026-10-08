//! Clocktable property-column parsing and row value projection.

use super::{
    ClockTableParameter, ClockTablePropertyColumns, ClockTablePropertyValue, ClockTableWarning,
    ClockTableWarningKind, ParsedAnnotation, Property, Section,
};

pub(super) fn clock_table_property_columns(
    parameters: &[ClockTableParameter],
) -> (Option<ClockTablePropertyColumns>, Vec<ClockTableWarning>) {
    if !parameter_present(parameters, "properties") {
        return (None, Vec::new());
    }

    let Some(raw) = parameter_value(parameters, "properties") else {
        return (
            None,
            vec![clock_table_properties_warning(
                "properties parameter has no property list",
            )],
        );
    };
    let Some(names) = parse_clock_table_property_names(&raw) else {
        return (
            None,
            vec![clock_table_properties_warning(
                "properties parameter is preserved but not applied; expected an Org list of property names",
            )],
        );
    };
    if names.is_empty() {
        return (None, Vec::new());
    }

    (
        Some(ClockTablePropertyColumns {
            names,
            inherit: clock_table_truthy_parameter(parameters, "inherit-props"),
        }),
        Vec::new(),
    )
}

pub(super) fn clock_table_property_values(
    section: &Section<ParsedAnnotation>,
    columns: &ClockTablePropertyColumns,
) -> Vec<ClockTablePropertyValue> {
    columns
        .names
        .iter()
        .map(|name| {
            if let Some(property) = find_property(&section.properties, name) {
                return ClockTablePropertyValue {
                    name: name.clone(),
                    value: Some(property.value.clone()),
                    inherited: false,
                };
            }

            let inherited = columns.inherit;
            let value = inherited
                .then(|| find_property(&section.effective_properties, name))
                .flatten()
                .map(|property| property.value.clone());
            let inherited = inherited && value.is_some();
            ClockTablePropertyValue {
                name: name.clone(),
                value,
                inherited,
            }
        })
        .collect()
}

fn parse_clock_table_property_names(raw: &str) -> Option<Vec<String>> {
    super::org_native_values::optional("clock-property-names", raw)
}
fn clock_table_truthy_parameter(parameters: &[ClockTableParameter], key: &str) -> bool {
    parameter_value(parameters, key)
        .is_some_and(|raw| super::org_native_values::scalar("parameter-truthy", &[&raw]) == "true")
}

fn parameter_value(parameters: &[ClockTableParameter], key: &str) -> Option<String> {
    parameters
        .iter()
        .find(|parameter| parameter.key.eq_ignore_ascii_case(key))
        .and_then(|parameter| parameter.value.clone())
}

fn parameter_present(parameters: &[ClockTableParameter], key: &str) -> bool {
    parameters
        .iter()
        .any(|parameter| parameter.key.eq_ignore_ascii_case(key))
}

fn clock_table_properties_warning(message: impl Into<String>) -> ClockTableWarning {
    ClockTableWarning {
        kind: ClockTableWarningKind::PropertiesPreserved,
        message: message.into(),
    }
}

fn find_property<'a>(
    properties: &'a [Property<ParsedAnnotation>],
    name: &str,
) -> Option<&'a Property<ParsedAnnotation>> {
    properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(name))
}
