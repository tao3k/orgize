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
        let rows = super::org_native_values::rows("agenda-match", &[expression.as_ref()]);
        let first = rows
            .first()
            .expect("native Agenda response must not be empty");
        if first.first().map(String::as_str) == Some("error") {
            assert_eq!(rows.len(), 1, "native Agenda error must be exclusive");
            let [_, position, message] = first.as_slice() else {
                panic!("invalid native Agenda error arity");
            };
            return Err(AgendaMatchParseError::new(
                position.parse().expect("native Agenda error byte position"),
                message.clone(),
            ));
        }
        let [tag, source] = first.as_slice() else {
            panic!("invalid native Agenda source arity");
        };
        assert_eq!(tag, "source", "invalid native Agenda source tag");
        let mut clauses: Vec<AgendaMatchClause> = Vec::new();
        for row in &rows[1..] {
            let [tag, index, positive, kind, key, operator, value_kind, value] = row.as_slice()
            else {
                panic!("invalid native Agenda term arity");
            };
            assert_eq!(tag, "term", "invalid native Agenda term tag");
            let index: usize = index.parse().expect("native Agenda clause index");
            if index == clauses.len() {
                clauses.push(AgendaMatchClause { terms: Vec::new() });
            }
            assert_eq!(
                index + 1,
                clauses.len(),
                "native Agenda clauses must be contiguous"
            );
            let positive = match positive.as_str() {
                "true" => true,
                "false" => false,
                _ => panic!("invalid native Agenda polarity"),
            };
            let predicate = match kind.as_str() {
                "tag" => {
                    assert!(operator.is_empty() && value_kind.is_empty() && value.is_empty());
                    AgendaMatchPredicate::Tag(key.clone())
                }
                "property" => AgendaMatchPredicate::Property {
                    key: key.clone(),
                    operator: match operator.as_str() {
                        "eq" => AgendaMatchOperator::Equal,
                        "ne" => AgendaMatchOperator::NotEqual,
                        "lt" => AgendaMatchOperator::Less,
                        "le" => AgendaMatchOperator::LessOrEqual,
                        "gt" => AgendaMatchOperator::Greater,
                        "ge" => AgendaMatchOperator::GreaterOrEqual,
                        _ => panic!("invalid native Agenda operator"),
                    },
                    value: match value_kind.as_str() {
                        "bare" => AgendaMatchValue::Bare(value.clone()),
                        "quoted" => AgendaMatchValue::Quoted(value.clone()),
                        "pattern" => AgendaMatchValue::Pattern(value.clone()),
                        _ => panic!("invalid native Agenda value kind"),
                    },
                },
                _ => panic!("invalid native Agenda predicate"),
            };
            clauses[index].terms.push(AgendaMatchTerm {
                positive,
                predicate,
            });
        }
        assert!(
            !clauses.is_empty(),
            "native Agenda success requires clauses"
        );
        Ok(Self {
            source: source.clone(),
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
