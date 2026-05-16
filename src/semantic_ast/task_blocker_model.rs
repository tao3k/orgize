//! Source-grounded task blocker evidence derived from native Org structures.

use super::{SectionIndexSource, TodoKeyword};

/// One blocker edge between two sibling TODO entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskBlockerRecord {
    pub kind: TaskBlockerKind,
    pub blocked: TaskBlockerTask,
    pub blocker: TaskBlockerTask,
    pub parent: TaskBlockerParent,
    pub message: String,
}

/// Stable blocker kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskBlockerKind {
    OrderedPreviousSibling,
}

impl TaskBlockerKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OrderedPreviousSibling => "orderedPreviousSibling",
        }
    }
}

/// The task that is blocked or blocking another task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskBlockerTask {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub todo: Option<TodoKeyword>,
}

/// The local parent entry whose `ORDERED` property created the blocker edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskBlockerParent {
    pub source: SectionIndexSource,
    pub ordered_property_source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
}
