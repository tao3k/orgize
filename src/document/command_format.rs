//! Shared document command formatting helpers.

use super::model::DocumentElement;

pub(super) fn heading_field<'a>(heading: &'a DocumentElement, key: &str) -> Option<&'a str> {
    heading
        .fields
        .iter()
        .find(|(field_key, _)| field_key == key)
        .map(|(_, value)| value.as_str())
}

pub(super) fn heading_fields<'a>(heading: &'a DocumentElement, key: &str) -> Vec<&'a str> {
    heading
        .fields
        .iter()
        .filter_map(|(field_key, value)| (field_key == key).then_some(value.as_str()))
        .collect()
}
