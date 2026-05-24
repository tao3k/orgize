//! Org Babel eval contract DTOs.

use super::{SourceBlockRecord, SourceBlockResultHandling};

/// Execution contract for one named Org Babel source block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BabelEvalPlan {
    pub name: String,
    pub record: SourceBlockRecord,
}

/// Error returned when a named Babel block cannot produce an eval contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BabelEvalPlanError {
    EmptyName,
    NotFound { name: String },
    Ambiguous { name: String, matches: usize },
}

/// Host-supplied execution output.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BabelEvalOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Text patch that inserts, replaces, or intentionally skips an Org Babel result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BabelEvalResultPatch {
    pub kind: BabelEvalResultPatchKind,
    pub range: Option<BabelEvalResultRange>,
    pub replacement: String,
    pub handling: SourceBlockResultHandling,
    pub message: Option<String>,
}

/// Patch operation kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BabelEvalResultPatchKind {
    Insert,
    Replace,
    Noop,
}

/// Byte range affected by a result patch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BabelEvalResultRange {
    pub start: u32,
    pub end: u32,
}

impl BabelEvalResultPatch {
    /// Applies the patch to an Org source string.
    #[must_use]
    pub fn apply_to(&self, source: &str) -> String {
        let Some(range) = self.range else {
            return source.to_string();
        };
        let start = usize::try_from(range.start)
            .unwrap_or(usize::MAX)
            .min(source.len());
        let end = usize::try_from(range.end)
            .unwrap_or(usize::MAX)
            .min(source.len())
            .max(start);
        let mut next = source.to_string();
        next.replace_range(start..end, self.replacement.as_str());
        next
    }
}
