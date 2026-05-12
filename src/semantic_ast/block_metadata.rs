//! Source and example block metadata parsing for semantic projection.

use super::{BlockCodeRef, BlockHeaderArg, BlockLineNumberMode, BlockLineNumbering};

pub(super) fn parse_block_line_numbering(switches: &str) -> Option<BlockLineNumbering> {
    let mut tokens = split_block_switches(switches).into_iter().peekable();
    let mut numbering = None;

    while let Some(token) = tokens.next() {
        let mode = match token.as_str() {
            "-n" => BlockLineNumberMode::New,
            "+n" => BlockLineNumberMode::Continued,
            _ => continue,
        };
        let start = tokens.peek().and_then(|value| value.parse::<usize>().ok());
        if start.is_some() {
            tokens.next();
        }
        numbering = Some(BlockLineNumbering { mode, start });
    }

    numbering
}

pub(super) fn parse_block_preserve_indentation(switches: &str) -> bool {
    split_block_switches(switches)
        .into_iter()
        .any(|token| token == "-i")
}

pub(super) fn parse_block_code_refs(value: &str, switches: Option<&str>) -> Vec<BlockCodeRef> {
    let label = switches
        .and_then(block_code_ref_label)
        .unwrap_or_else(default_code_ref_label);

    value
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            code_ref_in_line(line, &label).map(|(name, raw)| BlockCodeRef {
                line: index + 1,
                name,
                raw,
            })
        })
        .collect()
}

pub(super) fn parse_block_header_args(parameters: Option<&str>) -> Vec<BlockHeaderArg> {
    let Some(parameters) = parameters else {
        return Vec::new();
    };
    let starts = block_header_arg_starts(parameters);

    starts
        .iter()
        .enumerate()
        .filter_map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(parameters.len());
            block_header_arg_from_raw(&parameters[*start..end])
        })
        .collect()
}

fn block_code_ref_label(switches: &str) -> Option<CodeRefLabel> {
    let mut tokens = split_block_switches(switches).into_iter();

    while let Some(token) = tokens.next() {
        if token == "-l" {
            return tokens.next().and_then(CodeRefLabel::from_pattern);
        }
    }

    None
}

fn split_block_switches(switches: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = switches.chars().peekable();
    let mut in_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quote = !in_quote;
            }
            '\\' if in_quote => {
                if let Some(next) = chars.next() {
                    current.push(next);
                } else {
                    current.push(ch);
                }
            }
            ch if ch.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn block_header_arg_starts(parameters: &str) -> Vec<usize> {
    let mut scanner = BlockHeaderArgStartScanner::default();
    parameters
        .char_indices()
        .filter_map(|(index, ch)| scanner.next_start(parameters, index, ch))
        .collect()
}

#[derive(Default)]
struct BlockHeaderArgStartScanner {
    in_quote: bool,
    escaped: bool,
    prev: Option<char>,
}

impl BlockHeaderArgStartScanner {
    fn next_start(&mut self, parameters: &str, index: usize, ch: char) -> Option<usize> {
        if self.escaped {
            self.escaped = false;
            self.prev = Some(ch);
            return None;
        }

        let start = match ch {
            '\\' if self.in_quote => {
                self.escaped = true;
                None
            }
            '"' => {
                self.in_quote = !self.in_quote;
                None
            }
            ':' if self.is_header_arg_start(parameters, index, ch) => Some(index),
            _ => None,
        };

        self.prev = Some(ch);
        start
    }

    fn is_header_arg_start(&self, parameters: &str, index: usize, ch: char) -> bool {
        !self.in_quote
            && self.prev.is_none_or(char::is_whitespace)
            && parameters[index + ch.len_utf8()..]
                .chars()
                .next()
                .is_some_and(is_block_header_arg_key_char)
    }
}

fn block_header_arg_from_raw(raw: &str) -> Option<BlockHeaderArg> {
    let raw = raw.trim();
    let after_colon = raw.strip_prefix(':')?;
    let key_len = after_colon
        .chars()
        .take_while(|ch| is_block_header_arg_key_char(*ch))
        .map(char::len_utf8)
        .sum::<usize>();
    if key_len == 0 {
        return None;
    }

    let key = after_colon[..key_len].to_string();
    let value = after_colon[key_len..].trim();

    Some(BlockHeaderArg {
        key,
        value: (!value.is_empty()).then(|| value.to_string()),
        raw: raw.to_string(),
    })
}

fn is_block_header_arg_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodeRefLabel {
    prefix: String,
    suffix: String,
}

impl CodeRefLabel {
    fn from_pattern(pattern: String) -> Option<Self> {
        let (prefix, suffix) = pattern.split_once("%s")?;
        Some(Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
        })
    }
}

fn default_code_ref_label() -> CodeRefLabel {
    CodeRefLabel {
        prefix: "(ref:".into(),
        suffix: ")".into(),
    }
}

fn code_ref_in_line(line: &str, label: &CodeRefLabel) -> Option<(String, String)> {
    line.match_indices(&label.prefix)
        .filter_map(|(start, _)| code_ref_at(line, start, label))
        .last()
}

fn code_ref_at(line: &str, start: usize, label: &CodeRefLabel) -> Option<(String, String)> {
    if start > 0 && !line[..start].chars().next_back()?.is_whitespace() {
        return None;
    }

    let name_start = start + label.prefix.len();
    let suffix_start = if label.suffix.is_empty() {
        name_start
            + line[name_start..]
                .char_indices()
                .find_map(|(offset, ch)| (!is_code_ref_name_char(ch)).then_some(offset))
                .unwrap_or(line.len() - name_start)
    } else {
        name_start + line[name_start..].find(&label.suffix)?
    };
    let name = &line[name_start..suffix_start];

    if name.is_empty() || !name.chars().all(is_code_ref_name_char) {
        return None;
    }

    let end = suffix_start + label.suffix.len();
    Some((name.to_string(), line[start..end].to_string()))
}

fn is_code_ref_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' '
}
