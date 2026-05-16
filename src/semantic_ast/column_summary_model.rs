//! Non-mutating Column View summary plans.

use super::{ColumnViewColumn, ColumnViewRecord, SectionIndexSource};

/// Non-mutating plan for one `COLUMNS` declaration and its summary values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnSummaryPlan {
    pub declaration: ColumnViewRecord,
    pub rows: Vec<ColumnSummaryRow>,
    pub summaries: Vec<ColumnSummaryResult>,
    pub warnings: Vec<ColumnSummaryWarning>,
}

/// One section row visible to a Column View summary plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnSummaryRow {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub cells: Vec<ColumnSummaryCell>,
    pub summaries: Vec<ColumnSummaryResult>,
    pub children: Vec<ColumnSummaryRow>,
}

/// One collected value for a Column View property in a section row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnSummaryCell {
    pub property: String,
    pub value: Option<String>,
    pub source: ColumnSummaryValueSource,
}

/// Where a Column View cell value came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnSummaryValueSource {
    SpecialProperty,
    LocalProperty,
    InheritedProperty,
    Missing,
}

impl ColumnSummaryValueSource {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SpecialProperty => "specialProperty",
            Self::LocalProperty => "localProperty",
            Self::InheritedProperty => "inheritedProperty",
            Self::Missing => "missing",
        }
    }
}

/// Summary result for one column with a `{SUMMARY}` operator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnSummaryResult {
    pub column: ColumnViewColumn,
    pub operator: String,
    pub kind: ColumnSummaryOperatorKind,
    pub format: Option<String>,
    pub value: Option<String>,
    pub input_count: usize,
    pub parsed_input_count: usize,
    pub status: ColumnSummaryStatus,
}

/// Supported and preserved Org Column View summary operator categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnSummaryOperatorKind {
    NumericSum,
    Currency,
    NumericMin,
    NumericMax,
    NumericMean,
    CheckboxState,
    CheckboxCount,
    CheckboxPercent,
    DurationSum,
    DurationMin,
    DurationMax,
    DurationMean,
    AgeMin,
    AgeMax,
    AgeMean,
    Estimate,
    Custom,
}

impl ColumnSummaryOperatorKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NumericSum => "numericSum",
            Self::Currency => "currency",
            Self::NumericMin => "numericMin",
            Self::NumericMax => "numericMax",
            Self::NumericMean => "numericMean",
            Self::CheckboxState => "checkboxState",
            Self::CheckboxCount => "checkboxCount",
            Self::CheckboxPercent => "checkboxPercent",
            Self::DurationSum => "durationSum",
            Self::DurationMin => "durationMin",
            Self::DurationMax => "durationMax",
            Self::DurationMean => "durationMean",
            Self::AgeMin => "ageMin",
            Self::AgeMax => "ageMax",
            Self::AgeMean => "ageMean",
            Self::Estimate => "estimate",
            Self::Custom => "custom",
        }
    }
}

/// Computation status for a summary result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnSummaryStatus {
    Computed,
    NoInputs,
    IgnoredSpecialProperty,
    UnparsedInputs,
    Unsupported,
}

impl ColumnSummaryStatus {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Computed => "computed",
            Self::NoInputs => "noInputs",
            Self::IgnoredSpecialProperty => "ignoredSpecialProperty",
            Self::UnparsedInputs => "unparsedInputs",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Non-fatal warning produced while building a Column View summary plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnSummaryWarning {
    pub kind: ColumnSummaryWarningKind,
    pub message: String,
}

/// Stable warning kind for Column View summary plans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnSummaryWarningKind {
    MissingSectionScope,
}

impl ColumnSummaryWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingSectionScope => "missingSectionScope",
        }
    }
}
