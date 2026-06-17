//! Org Agenda-style tag/property match expression parsing.

use std::{error::Error, fmt, str::FromStr};

/// Parsed Org Agenda-style tag/property match expression.
///
/// This intentionally covers the common official syntax used by agenda tag
/// searches: `+tag`, `-tag`, `tag|other`, `PROP="value"`, and numeric
/// comparisons such as `Effort<2`. Parentheses remain outside this parser-v2
/// surface; document projections apply `#+TAGS:` group expansion when a tag
/// vocabulary is available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaMatchQuery {
    pub(crate) source: String,
    pub(crate) clauses: Vec<AgendaMatchClause>,
}

impl AgendaMatchQuery {
    /// Parses an Org Agenda-style tag/property match expression.
    pub fn parse(expression: impl AsRef<str>) -> Result<Self, AgendaMatchParseError> {
        let source = expression.as_ref().trim();
        if source.is_empty() {
            return Err(AgendaMatchParseError::new(0, "match expression is empty"));
        }

        let clauses = split_top_level(source, b'|')
            .into_iter()
            .map(|(position, clause)| parse_clause(clause, position))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            source: source.to_string(),
            clauses,
        })
    }

    /// Returns the original normalized expression text.
    pub fn expression(&self) -> &str {
        &self.source
    }
}

impl FromStr for AgendaMatchQuery {
    type Err = AgendaMatchParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Error returned when parsing an agenda match expression fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaMatchParseError {
    pub position: usize,
    pub message: String,
}

impl AgendaMatchParseError {
    fn new(position: usize, message: impl Into<String>) -> Self {
        Self {
            position,
            message: message.into(),
        }
    }
}

impl fmt::Display for AgendaMatchParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid agenda match expression at byte {}: {}",
            self.position, self.message
        )
    }
}

impl Error for AgendaMatchParseError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AgendaMatchClause {
    pub(crate) terms: Vec<AgendaMatchTerm>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AgendaMatchTerm {
    pub(crate) positive: bool,
    pub(crate) predicate: AgendaMatchPredicate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AgendaMatchPredicate {
    Tag(String),
    Property {
        key: String,
        operator: AgendaMatchOperator,
        value: AgendaMatchValue,
    },
}

/// Comparison operator for agenda property match expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaMatchOperator {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AgendaMatchValue {
    Bare(String),
    Quoted(String),
    Pattern(String),
}

impl AgendaMatchValue {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Bare(value) | Self::Quoted(value) | Self::Pattern(value) => value,
        }
    }

    pub(crate) fn is_pattern(&self) -> bool {
        matches!(self, Self::Pattern(_))
    }
}

fn parse_clause(
    clause: &str,
    base_position: usize,
) -> Result<AgendaMatchClause, AgendaMatchParseError> {
    let bytes = clause.as_bytes();
    let mut terms = Vec::new();
    let mut cursor = 0;

    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b'&')
        {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let mut positive = true;
        match bytes[cursor] {
            b'+' => cursor += 1,
            b'-' => {
                positive = false;
                cursor += 1;
            }
            _ => {}
        }

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        let start = cursor;
        cursor = scan_term_end(bytes, cursor);
        let raw = clause[start..cursor].trim();
        if raw.is_empty() {
            return Err(AgendaMatchParseError::new(
                base_position + start,
                "empty match term",
            ));
        }

        terms.push(AgendaMatchTerm {
            positive,
            predicate: parse_predicate(raw, base_position + start)?,
        });
    }

    if terms.is_empty() {
        Err(AgendaMatchParseError::new(
            base_position,
            "match clause is empty",
        ))
    } else {
        Ok(AgendaMatchClause { terms })
    }
}

fn parse_predicate(
    raw: &str,
    position: usize,
) -> Result<AgendaMatchPredicate, AgendaMatchParseError> {
    if let Some((operator_start, operator, operator_len)) = find_property_operator(raw) {
        let key = raw[..operator_start].trim();
        let value_start = operator_start + operator_len;
        let value = raw[value_start..].trim();
        if key.is_empty() {
            return Err(AgendaMatchParseError::new(
                position,
                "property match is missing a key",
            ));
        }
        if value.is_empty() {
            return Err(AgendaMatchParseError::new(
                position + value_start,
                "property match is missing a value",
            ));
        }
        Ok(AgendaMatchPredicate::Property {
            key: key.to_string(),
            operator,
            value: parse_value(value),
        })
    } else {
        Ok(AgendaMatchPredicate::Tag(raw.to_string()))
    }
}

fn parse_value(raw: &str) -> AgendaMatchValue {
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        AgendaMatchValue::Quoted(raw[1..raw.len() - 1].to_string())
    } else if raw.len() >= 2 && raw.starts_with('{') && raw.ends_with('}') {
        AgendaMatchValue::Pattern(raw[1..raw.len() - 1].to_string())
    } else {
        AgendaMatchValue::Bare(raw.to_string())
    }
}

fn find_property_operator(raw: &str) -> Option<(usize, AgendaMatchOperator, usize)> {
    let bytes = raw.as_bytes();
    let mut index = 0;
    let mut in_quote = false;
    let mut brace_depth = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'"' if brace_depth == 0 => in_quote = !in_quote,
            b'{' if !in_quote => brace_depth = brace_depth.saturating_add(1),
            b'}' if !in_quote => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }

        if !in_quote
            && brace_depth == 0
            && let Some((operator, len)) = operator_at(&raw[index..])
        {
            let len = if raw[index + len..].starts_with('*') {
                len + 1
            } else {
                len
            };
            return Some((index, operator, len));
        }
        index += 1;
    }

    None
}

fn operator_at(raw: &str) -> Option<(AgendaMatchOperator, usize)> {
    [
        ("<=", AgendaMatchOperator::LessOrEqual),
        (">=", AgendaMatchOperator::GreaterOrEqual),
        ("<>", AgendaMatchOperator::NotEqual),
        ("!=", AgendaMatchOperator::NotEqual),
        ("/=", AgendaMatchOperator::NotEqual),
        ("==", AgendaMatchOperator::Equal),
        ("=", AgendaMatchOperator::Equal),
        ("<", AgendaMatchOperator::Less),
        (">", AgendaMatchOperator::Greater),
    ]
    .into_iter()
    .find_map(|(needle, operator)| raw.starts_with(needle).then_some((operator, needle.len())))
}

fn split_top_level(input: &str, separator: u8) -> Vec<(usize, &str)> {
    let bytes = input.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let mut in_quote = false;
    let mut brace_depth = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'"' if brace_depth == 0 => in_quote = !in_quote,
            b'{' if !in_quote => brace_depth = brace_depth.saturating_add(1),
            b'}' if !in_quote => brace_depth = brace_depth.saturating_sub(1),
            byte if byte == separator && !in_quote && brace_depth == 0 => {
                parts.push((start, input[start..index].trim()));
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    parts.push((start, input[start..].trim()));
    parts
}

fn scan_term_end(bytes: &[u8], mut cursor: usize) -> usize {
    let mut in_quote = false;
    let mut brace_depth = 0usize;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' if brace_depth == 0 => in_quote = !in_quote,
            b'{' if !in_quote => brace_depth = brace_depth.saturating_add(1),
            b'}' if !in_quote => brace_depth = brace_depth.saturating_sub(1),
            b'+' | b'-' | b'&' if !in_quote && brace_depth == 0 => break,
            _ => {}
        }
        cursor += 1;
    }

    cursor
}
