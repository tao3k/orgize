//! Preprocessing directive helpers for semantic projection.

use rowan::TextRange;

use super::{IncludeDirective, IncludeOption, Keyword, MacroDefinition, ParsedAnnotation};

pub(super) fn include_directive(
    keyword: Keyword<ParsedAnnotation>,
) -> Result<IncludeDirective<ParsedAnnotation>, (TextRange, String)> {
    let range = keyword.ann.range;
    let raw_value = keyword.value.clone();
    let tokens = preprocessing_value_tokens(raw_value.trim());
    let Some(path) = tokens.first() else {
        return Err((range, "INCLUDE keyword is missing an include path".into()));
    };

    let mut arguments = Vec::new();
    let mut options = Vec::new();
    let mut index = 1;
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
            options.push(IncludeOption {
                key: key.to_string(),
                value,
                raw,
            });
        } else {
            arguments.push(token.value.clone());
        }
        index += 1;
    }

    Ok(IncludeDirective {
        ann: keyword.ann,
        path: path.value.clone(),
        raw_path: path.raw.clone(),
        arguments,
        options,
        raw_value,
    })
}

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

#[derive(Debug)]
struct PreprocessingValueToken {
    raw: String,
    value: String,
}

fn preprocessing_value_tokens(value: &str) -> Vec<PreprocessingValueToken> {
    PreprocessingValueTokenizer::new(value).collect()
}

struct PreprocessingValueTokenizer<'a> {
    value: &'a str,
    cursor: usize,
}

impl<'a> PreprocessingValueTokenizer<'a> {
    fn new(value: &'a str) -> Self {
        Self { value, cursor: 0 }
    }

    fn peek_char(&self) -> Option<char> {
        self.value[self.cursor..].chars().next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(char::is_whitespace) {
            let _ = self.advance_char();
        }
    }

    fn quoted_token(&mut self, start: usize, quote: char) -> PreprocessingValueToken {
        let mut token_value = String::new();
        let mut escaped = false;
        let _ = self.advance_char();

        while let Some(ch) = self.advance_char() {
            if escaped {
                token_value.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                break;
            } else {
                token_value.push(ch);
            }
        }
        if escaped {
            token_value.push('\\');
        }

        PreprocessingValueToken {
            raw: self.value[start..self.cursor].to_string(),
            value: token_value,
        }
    }

    fn bare_token(&mut self, start: usize) -> PreprocessingValueToken {
        while self.peek_char().is_some_and(|ch| !ch.is_whitespace()) {
            let _ = self.advance_char();
        }
        let raw = &self.value[start..self.cursor];
        PreprocessingValueToken {
            raw: raw.to_string(),
            value: raw.to_string(),
        }
    }
}

impl Iterator for PreprocessingValueTokenizer<'_> {
    type Item = PreprocessingValueToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let start = self.cursor;
        let first = self.peek_char()?;
        Some(if matches!(first, '"' | '\'') {
            self.quoted_token(start, first)
        } else {
            self.bare_token(start)
        })
    }
}

fn is_valid_macro_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
