//! Parser-v2 semantic helpers for keyword-backed document settings.

use super::{
    ExportSettings, FileLink, FileLinkPathKind, Keyword, KeywordAttribute, LinkAbbreviation,
    LinkSearch, LinkSearchKind, ParsedAnnotation,
};

pub(super) fn is_parsed_keyword(key: &str) -> bool {
    matches!(
        key.to_ascii_uppercase().as_str(),
        "TITLE" | "AUTHOR" | "DATE" | "CAPTION"
    )
}

pub(super) fn keyword_attributes(key: &str, value: &str) -> Vec<KeywordAttribute> {
    if !key.to_ascii_uppercase().starts_with("ATTR_") {
        return Vec::new();
    }

    let mut attributes = Vec::new();
    let tokens = shellish_tokens(value.trim());
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(key) = token.value.strip_prefix(':').filter(|key| !key.is_empty()) {
            let mut raw = token.raw.clone();
            let mut value = None;
            if tokens
                .get(index + 1)
                .is_some_and(|next| !next.value.starts_with(':'))
            {
                let next = &tokens[index + 1];
                raw.push(' ');
                raw.push_str(&next.raw);
                value = Some(next.value.clone());
                index += 1;
            }
            attributes.push(KeywordAttribute {
                key: key.to_string(),
                value,
                raw,
            });
        }
        index += 1;
    }
    attributes
}

pub(super) fn parse_tags(value: &str) -> Vec<String> {
    value
        .split(':')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn split_words(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn apply_options_keyword(value: &str, settings: &mut ExportSettings) {
    for token in value.split_whitespace() {
        let Some((key, value)) = token.split_once(':') else {
            continue;
        };
        match key {
            "H" => settings.headline_levels = value.parse().ok(),
            "-" => settings.special_strings = bool_option(value),
            "e" => settings.expand_entities = bool_option(value),
            _ => {}
        }
    }
}

pub(super) fn link_abbreviation(keyword: &Keyword<ParsedAnnotation>) -> Option<LinkAbbreviation> {
    let value = keyword.value.trim();
    let (name, replacement) = value.split_once(char::is_whitespace)?;
    Some(LinkAbbreviation {
        name: name.to_ascii_lowercase(),
        replacement: replacement.trim().to_string(),
        raw_value: keyword.value.clone(),
    })
}

pub(super) fn link_search(path: &str) -> Option<LinkSearch> {
    let (_, search) = path.split_once("::")?;
    link_search_suffix(search)
}

pub(super) fn link_search_suffix(search: &str) -> Option<LinkSearch> {
    let kind = if search.starts_with('*') {
        LinkSearchKind::Headline
    } else if search.starts_with('#') {
        LinkSearchKind::CustomId
    } else if search.starts_with('/') && search.ends_with('/') && search.len() > 1 {
        LinkSearchKind::Regexp
    } else if search.chars().all(|ch| ch.is_ascii_digit()) {
        LinkSearchKind::LineNumber
    } else {
        LinkSearchKind::Text
    };
    Some(LinkSearch {
        raw: search.to_string(),
        kind,
        normalized: normalized_link_search(search, kind),
    })
}

pub(super) fn file_link(path: &str, search: Option<LinkSearch>) -> Option<FileLink> {
    let (protocol, target) = path.split_once(':')?;
    if !is_file_link_protocol(protocol) {
        return None;
    }
    let file_path = target
        .split_once("::")
        .map(|(file_path, _)| file_path)
        .unwrap_or(target);
    Some(FileLink {
        protocol: protocol.to_string(),
        path: file_path.to_string(),
        path_kind: file_link_path_kind(file_path),
        search,
    })
}

fn is_file_link_protocol(protocol: &str) -> bool {
    matches!(
        protocol.to_ascii_lowercase().as_str(),
        "file" | "file+sys" | "file+emacs" | "file+shell"
    )
}

fn file_link_path_kind(path: &str) -> FileLinkPathKind {
    let path = path.trim();
    if path.is_empty() {
        FileLinkPathKind::Empty
    } else if path.starts_with("/ssh:") || path.starts_with("/scp:") {
        FileLinkPathKind::Remote
    } else if path.starts_with('/') {
        FileLinkPathKind::Absolute
    } else if path.starts_with("~/") {
        FileLinkPathKind::HomeRelative
    } else {
        FileLinkPathKind::Relative
    }
}

fn normalized_link_search(search: &str, kind: LinkSearchKind) -> String {
    match kind {
        LinkSearchKind::Headline => normalize_search_text(search.trim_start_matches('*')),
        LinkSearchKind::CustomId => search.trim_start_matches('#').trim().to_string(),
        LinkSearchKind::Regexp => search
            .strip_prefix('/')
            .and_then(|value| value.strip_suffix('/'))
            .unwrap_or(search)
            .trim()
            .to_string(),
        LinkSearchKind::LineNumber => search.trim().to_string(),
        LinkSearchKind::Text => normalize_search_text(search),
    }
}

fn normalize_search_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(super) fn expand_link_abbreviation(
    protocol: &str,
    path: &str,
    abbreviations: &[LinkAbbreviation],
) -> Option<String> {
    let abbreviation = abbreviations
        .iter()
        .find(|abbreviation| abbreviation.name.eq_ignore_ascii_case(protocol))?;
    let replacement = &abbreviation.replacement;
    if replacement.contains("%s") || replacement.contains("%h") {
        Some(
            replacement
                .replace("%s", path)
                .replace("%h", &percent_encode(path)),
        )
    } else {
        Some(format!("{replacement}{path}"))
    }
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

#[derive(Clone, Debug)]
struct ShellishToken {
    raw: String,
    value: String,
}

fn shellish_tokens(value: &str) -> Vec<ShellishToken> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while let Some(start) = next_shellish_token_start(value, cursor) {
        let (token, next) = shellish_token(value, start);
        tokens.push(token);
        cursor = next;
    }
    tokens
}

fn next_shellish_token_start(value: &str, cursor: usize) -> Option<usize> {
    value[cursor..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| cursor + offset)
}

fn shellish_token(value: &str, start: usize) -> (ShellishToken, usize) {
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
    (
        ShellishToken {
            raw: value[start..cursor].trim_end().to_string(),
            value: parsed,
        },
        cursor,
    )
}
