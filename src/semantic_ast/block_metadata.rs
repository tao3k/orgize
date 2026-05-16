//! Source and example block metadata parsing for semantic projection.

use super::{
    BlockCodeRef, BlockHeaderArg, BlockLine, BlockLineNumberMode, BlockLineNumbering, BlockSwitches,
};

pub(super) struct BlockLineOptions<'a> {
    pub(super) switches: &'a BlockSwitches,
    pub(super) tab_width: usize,
    pub(super) preserve_indentation: bool,
}

pub(super) fn parse_block_switches(switches: Option<&str>) -> BlockSwitches {
    let Some(raw) = switches else {
        return BlockSwitches::default();
    };

    let mut parsed = BlockSwitches {
        raw: Some(raw.to_string()),
        ..BlockSwitches::default()
    };
    let mut tokens = split_block_switches(raw).into_iter().peekable();

    while let Some(token) = tokens.next() {
        match token.as_str() {
            "-n" | "+n" => {
                let start = tokens.peek().and_then(|value| value.parse::<usize>().ok());
                if start.is_some() {
                    tokens.next();
                }
                parsed.line_numbering = Some(BlockLineNumbering {
                    mode: if token == "-n" {
                        BlockLineNumberMode::New
                    } else {
                        BlockLineNumberMode::Continued
                    },
                    start,
                });
            }
            "-i" => parsed.preserve_indentation = true,
            "-k" => parsed.keep_labels = true,
            "-r" => parsed.remove_labels = true,
            "-l" => parsed.label_format = tokens.next(),
            _ => {}
        }
    }

    parsed
}

pub(super) fn parse_block_lines<A>(
    value: &str,
    source: Option<&str>,
    options: BlockLineOptions<'_>,
    mut ann_for_line: impl FnMut(usize) -> A,
) -> Vec<BlockLine<A>> {
    let label = options
        .switches
        .label_format
        .clone()
        .and_then(|pattern| CodeRefLabel::from_pattern(&pattern))
        .unwrap_or_else(default_code_ref_label);
    let source_lines = source.map(split_block_lines).unwrap_or_default();

    let line_drafts = split_block_lines(value)
        .into_iter()
        .enumerate()
        .map(|(index, value_line)| {
            let number = index + 1;
            let source = source_lines
                .get(index)
                .map(|line| line.text)
                .unwrap_or(value_line.text);
            let code_ref = code_ref_in_line(value_line.text, &label).map(|code_ref| BlockCodeRef {
                line: number,
                column: code_ref.column,
                end_column: code_ref.end_column,
                name: code_ref.name,
                raw: code_ref.raw,
            });
            let value_without_code_ref = remove_code_ref(value_line.text, &label);
            let expanded_value = tabs_to_spaces(value_line.text, options.tab_width);

            BlockLineDraft {
                ann: ann_for_line(index),
                number,
                source: source.to_string(),
                value: value_line.text.to_string(),
                expanded_value,
                value_without_code_ref,
                line_ending: value_line.ending.map(ToString::to_string),
                code_ref,
            }
        })
        .collect::<Vec<_>>();

    let removed_indent = if options.preserve_indentation {
        0
    } else {
        line_drafts
            .iter()
            .map(|line| leading_spaces(&line.expanded_value))
            .min()
            .unwrap_or(0)
    };

    line_drafts
        .into_iter()
        .map(|line| {
            let normalized_value = drop_leading_spaces(&line.expanded_value, removed_indent);
            let normalized_value_without_code_ref = remove_code_ref(&normalized_value, &label);

            BlockLine {
                ann: line.ann,
                number: line.number,
                source: line.source,
                value: line.value,
                normalized_value,
                value_without_code_ref: line.value_without_code_ref,
                normalized_value_without_code_ref,
                removed_indent,
                line_ending: line.line_ending,
                code_ref: line.code_ref,
            }
        })
        .collect()
}

pub(super) fn parse_block_code_refs(value: &str, switches: Option<&str>) -> Vec<BlockCodeRef> {
    let switches = parse_block_switches(switches);

    block_code_refs(&parse_block_lines(
        value,
        None,
        BlockLineOptions {
            switches: &switches,
            tab_width: 4,
            preserve_indentation: switches.preserve_indentation,
        },
        |_| (),
    ))
}

pub(super) fn block_code_refs<A>(lines: &[BlockLine<A>]) -> Vec<BlockCodeRef> {
    lines
        .iter()
        .filter_map(|line| line.code_ref.clone())
        .collect()
}

pub(crate) fn parse_block_header_args(args: Option<&str>) -> Vec<BlockHeaderArg> {
    let Some(args) = args else {
        return Vec::new();
    };
    parse_keyword_args(args, ":")
}

fn parse_keyword_args(value: &str, prefix: &str) -> Vec<BlockHeaderArg> {
    let tokens = split_keyword_tokens_with_ranges(value);
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| block_header_arg(value, &tokens, index, token, prefix))
        .collect()
}

fn block_header_arg(
    source: &str,
    tokens: &[KeywordToken],
    index: usize,
    token: &KeywordToken,
    prefix: &str,
) -> Option<BlockHeaderArg> {
    let key = token
        .value
        .strip_prefix(prefix)
        .filter(|key| !key.is_empty())?;
    let end_index = next_keyword_arg_index(tokens, index + 1, prefix).unwrap_or(tokens.len());
    let end = tokens
        .get(end_index.saturating_sub(1))
        .map(|token| token.end)
        .unwrap_or(token.end);
    let value = (index + 1 < end_index)
        .then(|| source[tokens[index + 1].start..end].trim().to_string())
        .filter(|value| !value.is_empty());

    Some(BlockHeaderArg {
        key: key.to_string(),
        value,
        raw: source[token.start..end].trim().to_string(),
    })
}

fn next_keyword_arg_index(tokens: &[KeywordToken], start: usize, prefix: &str) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, token)| is_keyword_arg_token(token, prefix))
        .map(|(index, _)| index)
}

fn is_keyword_arg_token(token: &KeywordToken, prefix: &str) -> bool {
    token.value.starts_with(prefix) && token.value.len() > prefix.len()
}

fn split_keyword_tokens(value: &str) -> Vec<String> {
    split_keyword_tokens_with_ranges(value)
        .into_iter()
        .map(|token| token.value)
        .collect()
}

#[derive(Clone, Debug)]
struct KeywordToken {
    start: usize,
    end: usize,
    value: String,
}

fn split_keyword_tokens_with_ranges(value: &str) -> Vec<KeywordToken> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    while let Some(start) = next_keyword_token_start(value, cursor) {
        let (token, next) = keyword_token(value, start);
        tokens.push(token);
        cursor = next;
    }

    tokens
}

fn next_keyword_token_start(value: &str, cursor: usize) -> Option<usize> {
    value[cursor..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| cursor + offset)
}

fn keyword_token(value: &str, start: usize) -> (KeywordToken, usize) {
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
        KeywordToken {
            start,
            end: value[..cursor].trim_end().len(),
            value: parsed,
        },
        cursor,
    )
}

#[derive(Clone, Copy)]
pub(super) struct SplitBlockLine<'a> {
    pub(super) text: &'a str,
    pub(super) ending: Option<&'a str>,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn split_block_lines(value: &str) -> Vec<SplitBlockLine<'_>> {
    if value.is_empty() {
        return Vec::new();
    }

    let bytes = value.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                lines.push(SplitBlockLine {
                    text: &value[start..index],
                    ending: Some("\n"),
                    start,
                    end: index,
                });
                index += 1;
                start = index;
            }
            b'\r' if index + 1 < bytes.len() && bytes[index + 1] == b'\n' => {
                lines.push(SplitBlockLine {
                    text: &value[start..index],
                    ending: Some("\r\n"),
                    start,
                    end: index,
                });
                index += 2;
                start = index;
            }
            b'\r' => {
                lines.push(SplitBlockLine {
                    text: &value[start..index],
                    ending: Some("\r"),
                    start,
                    end: index,
                });
                index += 1;
                start = index;
            }
            _ => index += 1,
        }
    }

    if start < value.len() {
        lines.push(SplitBlockLine {
            text: &value[start..],
            ending: None,
            start,
            end: value.len(),
        });
    }

    lines
}

struct BlockLineDraft<A> {
    ann: A,
    number: usize,
    source: String,
    value: String,
    expanded_value: String,
    value_without_code_ref: String,
    line_ending: Option<String>,
    code_ref: Option<BlockCodeRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodeRefLabel {
    prefix: String,
    suffix: String,
}

impl CodeRefLabel {
    fn from_pattern(pattern: &str) -> Option<Self> {
        let marker = "%s";
        let marker_index = pattern.find(marker)?;
        Some(Self {
            prefix: pattern[..marker_index].to_string(),
            suffix: pattern[marker_index + marker.len()..].to_string(),
        })
    }
}

fn default_code_ref_label() -> CodeRefLabel {
    CodeRefLabel {
        prefix: "(ref:".to_string(),
        suffix: ")".to_string(),
    }
}

struct CodeRefMatch {
    end: usize,
    remove_start: usize,
    column: usize,
    end_column: usize,
    name: String,
    raw: String,
}

fn code_ref_in_line(line: &str, label: &CodeRefLabel) -> Option<CodeRefMatch> {
    for (index, _) in line.match_indices(&label.prefix) {
        if let Some(code_ref) = code_ref_at(line, index, label) {
            return Some(code_ref);
        }
    }

    None
}

fn code_ref_at(line: &str, index: usize, label: &CodeRefLabel) -> Option<CodeRefMatch> {
    let after_prefix = line.get(index + label.prefix.len()..)?;
    let suffix_offset = if label.suffix.is_empty() {
        after_prefix
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map(|(offset, _)| offset)
            .unwrap_or(after_prefix.len())
    } else {
        after_prefix.find(&label.suffix)?
    };
    let name = &after_prefix[..suffix_offset];
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':'))
    {
        return None;
    }

    let end = index + label.prefix.len() + suffix_offset + label.suffix.len();
    let mut before = line[..index].char_indices().rev();
    let remove_start = before
        .find_map(|(byte_index, ch)| (!ch.is_whitespace()).then_some(byte_index + ch.len_utf8()))
        .unwrap_or(0);
    let column = line[..index].chars().count() + 1;
    let end_column = column + line[index..end].chars().count();

    Some(CodeRefMatch {
        end,
        remove_start,
        column,
        end_column,
        name: name.to_string(),
        raw: line[index..end].to_string(),
    })
}

fn remove_code_ref(line: &str, label: &CodeRefLabel) -> String {
    let Some(code_ref) = code_ref_in_line(line, label) else {
        return line.to_string();
    };

    let mut value = String::new();
    value.push_str(&line[..code_ref.remove_start]);
    value.push_str(&line[code_ref.end..]);
    value
}

fn tabs_to_spaces(line: &str, tab_width: usize) -> String {
    let leading_width = line.chars().take_while(|ch| ch.is_whitespace()).count();
    let mut value = String::new();

    for ch in line.chars().take(leading_width) {
        if ch == '\t' {
            for _ in 0..tab_width {
                value.push(' ');
            }
        } else {
            value.push(ch);
        }
    }
    value.extend(line.chars().skip(leading_width));
    value
}

fn leading_spaces(value: &str) -> usize {
    value.chars().take_while(|ch| *ch == ' ').count()
}

fn drop_leading_spaces(value: &str, count: usize) -> String {
    value.chars().skip(count).collect()
}

fn split_block_switches(value: &str) -> Vec<String> {
    split_keyword_tokens(value)
}
