//! Backend-neutral HTML escaping and options for the Scheme AOT exporter.

use std::fmt;

/// Escape sensitive characters in HTML text or attributes.
pub struct HtmlEscape<S: AsRef<str>>(pub S);

impl<S: AsRef<str>> fmt::Display for HtmlEscape<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content = self.0.as_ref();
        let bytes = content.as_bytes();
        let mut pos = 0;
        while let Some(off) = jetscii::bytes!(b'<', b'>', b'&', b'\'', b'"').find(&bytes[pos..]) {
            f.write_str(&content[pos..pos + off])?;
            pos += off + 1;
            f.write_str(match bytes[pos - 1] {
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'&' => "&amp;",
                b'\'' => "&apos;",
                b'"' => "&quot;",
                _ => unreachable!(),
            })?;
        }
        f.write_str(&content[pos..])
    }
}

/// Options for graph-backed HTML export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlExportOptions {
    /// Convert Org special strings such as `--`, `---`, and `...`.
    pub special_strings: bool,
    /// Expand projected entity Objects to backend HTML.
    pub expand_entities: bool,
}

impl Default for HtmlExportOptions {
    fn default() -> Self {
        Self {
            special_strings: false,
            expand_entities: true,
        }
    }
}

pub(crate) fn safe_source_block_data_attributes(source: &str) -> Vec<(&str, &str)> {
    let mut attributes = Vec::new();
    for line in source.lines() {
        let line = line.trim_start();
        let prefix = "#+attr_html:";
        let Some(candidate) = line.get(..prefix.len()) else {
            continue;
        };
        if !candidate.eq_ignore_ascii_case(prefix) {
            continue;
        }

        let tokens = line[prefix.len()..].split_whitespace().collect::<Vec<_>>();
        let mut index = 0;
        while index < tokens.len() {
            let Some(name) = tokens[index].strip_prefix(':') else {
                index += 1;
                continue;
            };
            let value = tokens
                .get(index + 1)
                .filter(|value| !value.starts_with(':'))
                .copied()
                .unwrap_or("");
            if name.strip_prefix("data-").is_some_and(|suffix| {
                !suffix.is_empty()
                    && suffix.bytes().all(|byte| {
                        byte.is_ascii_lowercase()
                            || byte.is_ascii_digit()
                            || matches!(byte, b'-' | b'_' | b'.' | b':')
                    })
            }) {
                attributes.push((name, value.trim_matches(['\'', '"'])));
            }
            index += if value.is_empty() { 1 } else { 2 };
        }
    }
    attributes
}

pub(crate) fn special_strings(value: &str) -> String {
    value
        .replace("---", "\u{2014}")
        .replace("--", "\u{2013}")
        .replace("...", "\u{2026}")
        .replace("\\-", "\u{00AD}")
        .replace('\'', "\u{2019}")
}
