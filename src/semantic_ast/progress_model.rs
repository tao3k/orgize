//! Progress, effort, and dependency rollup records for planning consumers.

use super::{OrgDuration, SectionIndexSource};

/// One source-grounded progress rollup for an Org section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressStatsRecord {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub todo: ProgressTodoState,
    pub descendant_todos: ProgressTodoSummary,
    pub checkboxes: ProgressCheckboxSummary,
    pub statistic_cookies: Vec<ProgressStatisticCookie>,
    pub effort: ProgressEffortSummary,
    pub dependencies: Vec<TaskDependencyRecord>,
}

/// TODO state visible on the section itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressTodoState {
    None,
    Todo,
    Done,
}

impl ProgressTodoState {
    /// Stable label for compact and DTO consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Todo => "todo",
            Self::Done => "done",
        }
    }
}

/// Descendant TODO summary for one section.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProgressTodoSummary {
    pub total: u32,
    pub done: u32,
    pub open: u32,
}

/// Checkbox summary for one section subtree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProgressCheckboxSummary {
    pub total: u32,
    pub checked: u32,
    pub unchecked: u32,
    pub partial: u32,
}

impl ProgressCheckboxSummary {
    pub(crate) fn unresolved(self) -> u32 {
        self.unchecked + self.partial
    }
}

/// Parsed statistics cookie from a headline or section body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressStatisticCookie {
    pub source: SectionIndexSource,
    pub raw: String,
    pub kind: ProgressStatisticCookieKind,
    pub done: Option<u32>,
    pub total: Option<u32>,
    pub percent: Option<u8>,
}

/// Statistics cookie shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressStatisticCookieKind {
    Fraction,
    Percent,
    Unknown,
}

impl ProgressStatisticCookieKind {
    /// Stable label for compact and DTO consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fraction => "fraction",
            Self::Percent => "percent",
            Self::Unknown => "unknown",
        }
    }
}

/// Local and subtree effort rollup for one section.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProgressEffortSummary {
    pub local: Option<OrgDuration>,
    pub subtree_total_seconds: u64,
}

/// One dependency or blocker signal derived from native Org structures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskDependencyRecord {
    pub source: SectionIndexSource,
    pub kind: TaskDependencyKind,
    pub count: u32,
    pub message: String,
}

/// Dependency signal kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskDependencyKind {
    OpenDescendantTodo,
    OpenCheckbox,
    OrderedProperty,
}

impl TaskDependencyKind {
    /// Stable label for compact and DTO consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenDescendantTodo => "openDescendantTodo",
            Self::OpenCheckbox => "openCheckbox",
            Self::OrderedProperty => "orderedProperty",
        }
    }
}
