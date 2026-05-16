//! Dynamic block registry records for agent-facing Org projections.

use super::SectionIndexSource;

/// Non-executing projection for one native Org dynamic block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicBlockRecord {
    pub source: SectionIndexSource,
    pub name: String,
    pub writer: DynamicBlockWriterKind,
    pub parameters: Vec<DynamicBlockParameter>,
    pub content_state: DynamicBlockContentState,
    pub content_line_count: usize,
}

/// Stable writer category for known and custom dynamic blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicBlockWriterKind {
    ClockTable,
    ColumnView,
    Unknown,
}

impl DynamicBlockWriterKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClockTable => "clocktable",
            Self::ColumnView => "columnview",
            Self::Unknown => "unknown",
        }
    }
}

/// One native dynamic-block parameter from the begin line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicBlockParameter {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
}

/// Whether the dynamic block currently carries previously generated content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicBlockContentState {
    Empty,
    ExistingOutput,
}

impl DynamicBlockContentState {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::ExistingOutput => "existingOutput",
        }
    }
}
