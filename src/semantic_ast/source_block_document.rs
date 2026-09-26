//! Typed construction of agent-facing Org source-block documents.

use std::{error::Error, fmt};

use crate::org_aot::parse_org_aot;

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
        if headers.iter().enumerate().any(|(index, header)| {
            headers[..index]
                .iter()
                .any(|prior| prior.key.eq_ignore_ascii_case(&header.key))
        }) {
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
    let document = parse_org_aot(source)
        .map_err(|_| rendering("rendered Org did not parse as the expected document"))?;
    if document.syntax().to_string() != source {
        return Err(rendering("rendered Org did not round-trip through Rowan"));
    }
    let records = document.records();
    let Some(root) = records.first().filter(|record| record.kind == "org-data") else {
        return Err(rendering(
            "rendered Org did not parse as the expected document",
        ));
    };
    let expected_element_count = expected
        .iter()
        .map(|block| block.preamble_keywords.len() + 1)
        .sum::<usize>();
    if root.child_ids.len() != expected_element_count {
        return Err(rendering(
            "rendered Org did not parse as the expected document",
        ));
    }
    if records
        .iter()
        .filter(|record| record.kind == "src-block")
        .count()
        != expected.len()
    {
        return Err(rendering(
            "rendered Org did not parse as exactly the expected source blocks",
        ));
    }
    let mut elements = root.child_ids.iter().filter_map(|id| records.get(*id));
    for block in expected {
        for keyword in &block.preamble_keywords {
            let Some(element) = elements.next() else {
                return Err(rendering(
                    "rendered Org lost a source-block preamble keyword",
                ));
            };
            if element.kind != "keyword"
                || !element
                    .field("key")
                    .is_some_and(|key| key.eq_ignore_ascii_case(&keyword.key))
                || element.field("value").map(str::trim) != Some(keyword.value.as_str())
            {
                return Err(rendering(
                    "rendered Org source-block preamble is not an adjacent generic keyword",
                ));
            }
        }
        let Some(record) = elements.next() else {
            return Err(rendering("rendered Org lost a source block"));
        };
        if record.kind != "src-block" || record.field("language") != Some(block.language.as_str()) {
            return Err(rendering(
                "rendered Org source-block structure differs from its typed input",
            ));
        }
        let parsed_keys = record.values("header-key").collect::<Vec<_>>();
        let parsed_values = record.values("header-value").collect::<Vec<_>>();
        if parsed_keys.len() != block.headers.len() || parsed_values.len() != block.headers.len() {
            return Err(rendering(
                "rendered Org source-block header differs from its typed input",
            ));
        }
        for ((parsed_key, parsed_value), header) in parsed_keys
            .into_iter()
            .zip(parsed_values)
            .zip(&block.headers)
        {
            let mut expected_value = String::new();
            render_header_value(&mut expected_value, &header.value);
            if parsed_key != header.key || parsed_value != expected_value {
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
