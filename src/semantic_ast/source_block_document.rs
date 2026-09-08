//! Typed construction of agent-facing Org source-block documents.

use std::{error::Error, fmt};

use super::{BlockKind, ElementData};
use crate::Org;

/// One generic Org keyword emitted immediately before a source block.
///
/// This does not change Org's fixed affiliated-keyword vocabulary.  Consumers
/// own the meaning of the adjacency between this standalone keyword and the
/// following source block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSourceBlockKeyword {
    key: String,
    value: String,
}

impl OrgSourceBlockKeyword {
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, OrgSourceBlockDocumentError> {
        let key = key.into();
        let value = value.into();
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return Err(invalid("source-block preamble keyword key is invalid"));
        }
        if value.trim().is_empty() || value.contains(['\r', '\n']) {
            return Err(invalid(
                "source-block preamble keyword value must be nonempty and single-line",
            ));
        }
        Ok(Self {
            key: key.to_ascii_uppercase(),
            value,
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Whether one source-block header value is a bare token or quoted text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgSourceBlockHeaderValue {
    Token(String),
    Text(String),
}

impl OrgSourceBlockHeaderValue {
    pub fn token(value: impl Into<String>) -> Result<Self, OrgSourceBlockDocumentError> {
        let value = value.into();
        if value.is_empty()
            || value.bytes().any(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b'"' | b'\'' | b':' | b'\\')
            })
        {
            return Err(invalid("source-block token header value is invalid"));
        }
        Ok(Self::Token(value))
    }

    pub fn text(value: impl Into<String>) -> Result<Self, OrgSourceBlockDocumentError> {
        let value = value.into();
        if value.contains(['\r', '\n']) {
            return Err(invalid(
                "source-block text header value must be single-line",
            ));
        }
        Ok(Self::Text(value))
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Token(value) | Self::Text(value) => value,
        }
    }
}

/// One typed `:key value` source-block header argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSourceBlockHeader {
    key: String,
    value: OrgSourceBlockHeaderValue,
}

impl OrgSourceBlockHeader {
    pub fn new(
        key: impl Into<String>,
        value: OrgSourceBlockHeaderValue,
    ) -> Result<Self, OrgSourceBlockDocumentError> {
        let key = key.into();
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return Err(invalid("source-block header key is invalid"));
        }
        Ok(Self { key, value })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &OrgSourceBlockHeaderValue {
        &self.value
    }
}

/// One source block whose Org envelope is owned by Orgize.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSourceBlock {
    language: String,
    headers: Vec<OrgSourceBlockHeader>,
    preamble_keywords: Vec<OrgSourceBlockKeyword>,
    body: String,
}

impl OrgSourceBlock {
    pub fn new(
        language: impl Into<String>,
        headers: Vec<OrgSourceBlockHeader>,
        preamble_keywords: Vec<OrgSourceBlockKeyword>,
        body: impl Into<String>,
    ) -> Result<Self, OrgSourceBlockDocumentError> {
        let language = language.into();
        if language.is_empty()
            || !language
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+'))
        {
            return Err(invalid("source-block language is invalid"));
        }
        if headers
            .iter()
            .enumerate()
            .any(|(index, header)| headers[..index].iter().any(|prior| prior.key == header.key))
        {
            return Err(invalid("source-block header keys must be unique"));
        }
        if preamble_keywords
            .iter()
            .enumerate()
            .any(|(index, keyword)| {
                preamble_keywords[..index]
                    .iter()
                    .any(|prior| prior.key == keyword.key && prior.value == keyword.value)
            })
        {
            return Err(invalid("source-block preamble keywords must be unique"));
        }
        let body = body.into();
        if body.lines().any(|line| {
            line.trim_start()
                .get(..9)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+end_src"))
        }) {
            return Err(invalid(
                "source-block body cannot contain an Org source-block end marker",
            ));
        }
        Ok(Self {
            language,
            headers,
            preamble_keywords,
            body,
        })
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn headers(&self) -> &[OrgSourceBlockHeader] {
        &self.headers
    }

    pub fn preamble_keywords(&self) -> &[OrgSourceBlockKeyword] {
        &self.preamble_keywords
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

/// A nonempty Org document containing only typed source blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSourceBlockDocument {
    blocks: Vec<OrgSourceBlock>,
}

impl OrgSourceBlockDocument {
    pub fn new(blocks: Vec<OrgSourceBlock>) -> Result<Self, OrgSourceBlockDocumentError> {
        if blocks.is_empty() {
            return Err(invalid(
                "source-block document must contain at least one block",
            ));
        }
        Ok(Self { blocks })
    }

    pub fn blocks(&self) -> &[OrgSourceBlock] {
        &self.blocks
    }

    /// Render canonical Org and re-admit it through Orgize before exposing it.
    pub fn render(&self) -> Result<String, OrgSourceBlockDocumentError> {
        let mut output = String::new();
        for (index, block) in self.blocks.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            for keyword in &block.preamble_keywords {
                output.push_str("#+");
                output.push_str(&keyword.key);
                output.push_str(": ");
                output.push_str(&keyword.value);
                output.push('\n');
            }
            output.push_str("#+begin_src ");
            output.push_str(&block.language);
            for header in &block.headers {
                output.push_str(" :");
                output.push_str(&header.key);
                output.push(' ');
                render_header_value(&mut output, &header.value);
            }
            output.push('\n');
            output.push_str(&block.body);
            if !block.body.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("#+end_src\n");
        }
        admit_rendered_document(&output, &self.blocks)?;
        Ok(output)
    }
}

fn render_header_value(output: &mut String, value: &OrgSourceBlockHeaderValue) {
    match value {
        OrgSourceBlockHeaderValue::Token(value) => output.push_str(value),
        OrgSourceBlockHeaderValue::Text(value) => {
            output.push('"');
            for character in value.chars() {
                match character {
                    '"' => output.push_str("\\\""),
                    '\\' => output.push_str("\\\\"),
                    character => output.push(character),
                }
            }
            output.push('"');
        }
    }
}

fn admit_rendered_document(
    source: &str,
    expected: &[OrgSourceBlock],
) -> Result<(), OrgSourceBlockDocumentError> {
    let document = Org::parse(source).document();
    let expected_element_count = expected
        .iter()
        .map(|block| block.preamble_keywords.len() + 1)
        .sum::<usize>();
    if !document.diagnostics.is_empty() || document.children.len() != expected_element_count {
        return Err(rendering(
            "rendered Org did not parse as the expected document",
        ));
    }
    let records = document.source_block_records();
    if records.len() != expected.len() {
        return Err(rendering(
            "rendered Org did not parse as exactly the expected source blocks",
        ));
    }
    let mut elements = document.children.iter();
    for (record, block) in records.iter().zip(expected.iter()) {
        for keyword in &block.preamble_keywords {
            let Some(element) = elements.next() else {
                return Err(rendering(
                    "rendered Org lost a source-block preamble keyword",
                ));
            };
            if !matches!(
                &element.data,
                ElementData::Keyword(actual)
                    if actual.key.eq_ignore_ascii_case(&keyword.key)
                        && actual.value.trim() == keyword.value
            ) {
                return Err(rendering(
                    "rendered Org source-block preamble is not an adjacent generic keyword",
                ));
            }
        }
        let Some(element) = elements.next() else {
            return Err(rendering("rendered Org lost a source block"));
        };
        if !matches!(&element.data, ElementData::Block(parsed) if parsed.kind == BlockKind::Source)
            || record.language.as_deref() != Some(block.language.as_str())
        {
            return Err(rendering(
                "rendered Org source-block structure differs from its typed input",
            ));
        }
        for header in &block.headers {
            let parsed = record
                .normalized_header_args
                .iter()
                .find(|candidate| candidate.key == header.key)
                .and_then(|candidate| candidate.value.as_deref());
            let mut expected_value = String::new();
            render_header_value(&mut expected_value, &header.value);
            if parsed != Some(expected_value.as_str()) {
                return Err(rendering(
                    "rendered Org source-block header differs from its typed input",
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgSourceBlockDocumentError {
    reason_kind: &'static str,
    message: String,
}

impl OrgSourceBlockDocumentError {
    pub fn reason_kind(&self) -> &'static str {
        self.reason_kind
    }
}

impl fmt::Display for OrgSourceBlockDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason_kind, self.message)
    }
}

impl Error for OrgSourceBlockDocumentError {}

fn invalid(message: impl Into<String>) -> OrgSourceBlockDocumentError {
    OrgSourceBlockDocumentError {
        reason_kind: "org-source-block-input-invalid",
        message: message.into(),
    }
}

fn rendering(message: impl Into<String>) -> OrgSourceBlockDocumentError {
    OrgSourceBlockDocumentError {
        reason_kind: "org-source-block-rendering-invalid",
        message: message.into(),
    }
}
