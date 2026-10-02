//! Parser-v2 semantic helpers for keyword-backed document settings.

use super::{ExportSettings, Keyword, LinkAbbreviation, ParsedAnnotation};

pub(super) fn parse_tags(value: &str) -> Vec<String> {
    crate::org_aot::keyword_tag_words(value)
}

pub(super) fn split_words(value: &str) -> Vec<String> {
    crate::org_aot::keyword_words(value)
}

pub(super) fn apply_options_keyword(value: &str, settings: &mut ExportSettings) {
    let levels = crate::org_aot::keyword_option_value(value, "H");
    if crate::org_aot::keyword_option_present(value, "H") {
        settings.headline_levels = levels.parse().ok();
    }
    let special_strings = crate::org_aot::keyword_option_value(value, "-");
    if crate::org_aot::keyword_option_present(value, "-") {
        settings.special_strings = crate::org_aot::keyword_boolean_value(&special_strings);
    }
    let expand_entities = crate::org_aot::keyword_option_value(value, "e");
    if crate::org_aot::keyword_option_present(value, "e") {
        settings.expand_entities = crate::org_aot::keyword_boolean_value(&expand_entities);
    }
}

pub(super) fn link_abbreviation(keyword: &Keyword<ParsedAnnotation>) -> Option<LinkAbbreviation> {
    let name = crate::org_aot::keyword_first_word(&keyword.value);
    let replacement = crate::org_aot::keyword_rest(&keyword.value);
    if name.is_empty() || replacement.is_empty() {
        return None;
    }
    Some(LinkAbbreviation {
        name: name.to_ascii_lowercase(),
        replacement,
        raw_value: keyword.value.clone(),
    })
}

pub(super) fn expand_link_abbreviation(
    protocol: &str,
    path: &str,
    abbreviations: &[LinkAbbreviation],
) -> Option<String> {
    let abbreviation = abbreviations
        .iter()
        .find(|abbreviation| abbreviation.name.eq_ignore_ascii_case(protocol))?;
    Some(crate::org_aot::org_expand_link_abbreviation(
        &abbreviation.replacement,
        path,
        &percent_encode(path),
    ))
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
