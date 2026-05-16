//! Include expansion planning records.

use super::{IncludeDirective, IncludeOption};

/// Options for building an include expansion plan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IncludeExpansionOptions {
    pub base_dir: Option<String>,
}

impl IncludeExpansionOptions {
    /// Creates options that resolve relative include paths against a base directory.
    pub fn with_base_dir(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }
}

/// A safe, non-executing include expansion plan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IncludeExpansionPlan<A = ()> {
    pub entries: Vec<IncludeExpansionEntry<A>>,
}

/// One include directive normalized for explicit expansion by callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncludeExpansionEntry<A = ()> {
    pub directive: IncludeDirective<A>,
    pub resolved_path: Option<String>,
    pub line_selection: IncludeLineSelection,
    pub min_level: Option<usize>,
    pub mode: IncludeExpansionMode,
    pub options: Vec<IncludeOption>,
}

/// Parsed `:lines` selection from an include directive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncludeLineSelection {
    All,
    Range {
        start: Option<usize>,
        end: Option<usize>,
        raw: String,
    },
    Invalid {
        raw: String,
    },
}

/// Presentation mode requested by include arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IncludeExpansionMode {
    Org,
    Example,
    Source { language: Option<String> },
    Export { backend: Option<String> },
    Other { arguments: Vec<String> },
}
