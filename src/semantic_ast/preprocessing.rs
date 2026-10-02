//! Preprocessing directive helpers for semantic projection.

use rowan::TextRange;

use super::{Keyword, MacroDefinition, ParsedAnnotation};

pub(super) fn macro_definition(
    keyword: Keyword<ParsedAnnotation>,
) -> Result<MacroDefinition<ParsedAnnotation>, (TextRange, String)> {
    let range = keyword.ann.range;
    let raw_value = keyword.value.clone();
    let value = raw_value.trim_start();
    let name_end = value.find(char::is_whitespace).unwrap_or(value.len());
    let name = &value[..name_end];

    if !is_valid_macro_name(name) {
        return Err((range, "MACRO keyword is missing a valid macro name".into()));
    }

    Ok(MacroDefinition {
        ann: keyword.ann,
        name: name.to_string(),
        template: value[name_end..].trim_start().to_string(),
        raw_value,
    })
}

pub(super) fn split_macro_args(args: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in args.chars() {
        if escaped {
            if ch != ',' && ch != '\\' {
                current.push('\\');
            }
            current.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == ',' {
            let value = current.trim();
            if !value.is_empty() {
                values.push(value.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    if escaped {
        current.push('\\');
    }

    let value = current.trim();
    if !value.is_empty() {
        values.push(value.to_string());
    }

    values
}

fn is_valid_macro_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
