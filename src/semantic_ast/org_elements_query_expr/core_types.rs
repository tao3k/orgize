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

pub(super) use crate::org_aot::NativeExpressionValue as QueryExpr;

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

// Fixed-arity lowering contracts shared by query and predicate projections.
// Variadic combinators/selectors retain their existing owning compilers.
pub(super) fn query_form_arity_is_valid(head: &str, length: usize) -> bool {
    match head {
        "not"
        | "predicate"
        | "positive-integer"
        | "kind"
        | "type"
        | "category"
        | "affiliated-name"
        | "affiliatedName"
        | "name"
        | "context"
        | "outline-path-prefix"
        | "outlinePathPrefix"
        | "outline-path-exact-len"
        | "outlinePathExactLen"
        | "outline-depth"
        | "limit"
        | "source-path"
        | "source-path-contains"
        | "source-filename"
        | "source-filename-prefix"
        | "source-filename-suffix"
        | "source-filename-stem-uppercase" => length == 2,
        "=" | "contains" | "summary" | "summary-contains" | "property" | "property-contains" => {
            length == 3
        }
        _ => true,
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
