//! Parser-v2 semantic helpers for keyword-backed document settings.

use super::{
    ExportSettings, Keyword, LinkAbbreviation, ParsedAnnotation, TagDefinition, TagDefinitionGroup,
};

pub(super) fn parse_tags(value: &str) -> Vec<String> {
    crate::org_aot::keyword_tag_words(value)
}

pub(super) fn parse_tag_definitions(value: &str) -> Vec<TagDefinition> {
    let mut definitions: Vec<TagDefinition> = Vec::new();
    let words = crate::org_aot::keyword_words(value);
    let tokens = words.iter().map(String::as_str).collect::<Vec<_>>();
    let mut group_stack: Vec<TagGroupState> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match *token {
            "{" | "[" => {
                group_stack.push(TagGroupState {
                    exclusive: *token == "{",
                    labeled: group_has_separator(&tokens[index + 1..], closing_delimiter(token)),
                    parent: None,
                    after_separator: false,
                });
                continue;
            }
            "}" | "]" => {
                group_stack.pop();
                continue;
            }
            ":" => {
                if let Some(group) = group_stack.last_mut() {
                    group.after_separator = true;
                }
                continue;
            }
            _ => {}
        }

        if let Some(shortcut) = shortcut_token(token) {
            if let Some(previous) = definitions.last_mut()
                && previous.shortcut.is_none()
            {
                previous.shortcut = Some(shortcut.to_string());
                previous.raw.push(' ');
                previous.raw.push_str(token);
            }
            continue;
        }

        let (name, shortcut) = split_tag_shortcut(token);
        if name.is_empty() {
            continue;
        }
        let active_group = group_stack.last_mut();
        let is_group = active_group
            .as_ref()
            .is_some_and(|group| group.labeled && !group.after_separator);
        let group = active_group.as_ref().and_then(|group| {
            if is_group {
                None
            } else {
                Some(TagDefinitionGroup {
                    name: group.parent.clone(),
                    exclusive: group.exclusive,
                })
            }
        });
        definitions.push(TagDefinition {
            name: name.to_string(),
            shortcut: shortcut.map(ToString::to_string),
            raw: token.to_string(),
            is_group,
            group,
        });
        if is_group && let Some(group) = group_stack.last_mut() {
            group.parent = Some(name.to_string());
        }
    }
    definitions
}

pub(super) fn split_words(value: &str) -> Vec<String> {
    crate::org_aot::keyword_words(value)
}

fn shortcut_token(token: &str) -> Option<&str> {
    token
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .filter(|value| !value.is_empty())
}

#[derive(Debug)]
struct TagGroupState {
    exclusive: bool,
    labeled: bool,
    parent: Option<String>,
    after_separator: bool,
}

fn closing_delimiter(open: &str) -> &str {
    if open == "{" { "}" } else { "]" }
}

fn group_has_separator(tokens: &[&str], closing: &str) -> bool {
    tokens
        .iter()
        .take_while(|token| **token != closing)
        .any(|token| *token == ":")
}

fn split_tag_shortcut(token: &str) -> (&str, Option<&str>) {
    let Some(shortcut_end) = token.strip_suffix(')') else {
        return (token, None);
    };
    let Some(open) = shortcut_end.rfind('(') else {
        return (token, None);
    };
    let name = token[..open].trim();
    let shortcut = shortcut_end[open + 1..].trim();
    if name.is_empty() || shortcut.is_empty() {
        (token, None)
    } else {
        (name, Some(shortcut))
    }
}

pub(super) fn apply_options_keyword(value: &str, settings: &mut ExportSettings) {
    let levels = crate::org_aot::keyword_option_value(value, "H");
    if crate::org_aot::keyword_option_present(value, "H") {
        settings.headline_levels = levels.parse().ok();
    }
    let special_strings = crate::org_aot::keyword_option_value(value, "-");
    if crate::org_aot::keyword_option_present(value, "-") {
        settings.special_strings = bool_option(&special_strings);
    }
    let expand_entities = crate::org_aot::keyword_option_value(value, "e");
    if crate::org_aot::keyword_option_present(value, "e") {
        settings.expand_entities = bool_option(&expand_entities);
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

fn bool_option(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "t" | "true" | "yes" => Some(true),
        "nil" | "false" | "no" => Some(false),
        _ => None,
    }
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
