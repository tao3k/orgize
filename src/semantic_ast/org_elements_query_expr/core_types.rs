//! Query expression AST, field refs, errors, and enum boundaries.

use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsQueryExpressionError {
    message: String,
}

impl OrgElementsQueryExpressionError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for OrgElementsQueryExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for OrgElementsQueryExpressionError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum QueryExpr {
    Atom(String),
    String(String),
    List(Vec<QueryExpr>),
}

impl QueryExpr {
    pub(super) fn as_atom(&self) -> Option<&str> {
        match self {
            Self::Atom(value) => Some(value),
            Self::String(_) | Self::List(_) => None,
        }
    }

    pub(super) fn as_text(&self) -> Option<String> {
        match self {
            Self::Atom(value) | Self::String(value) => Some(value.clone()),
            Self::List(_) => None,
        }
    }

    pub(super) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Atom(value) => match value.as_str() {
                "t" | "true" => Some(true),
                "nil" | "false" => Some(false),
                _ => None,
            },
            Self::String(_) | Self::List(_) => None,
        }
    }
}

/// Parses an elisp-style Org elements query expression into the shared index

pub(super) fn list_head(items: &[QueryExpr]) -> Option<&str> {
    items.first()?.as_atom()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FieldKind {
    Summary,
    Property,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FieldRef {
    pub(super) kind: FieldKind,
    pub(super) key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RelativeKind {
    Descendant,
    Child,
    At,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentTextPredicateKind {
    PathEquals,
    PathContains,
    FilenameEquals,
    FilenamePrefix,
    FilenameSuffix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentBoolPredicateKind {
    FilenameStemUppercase,
}
