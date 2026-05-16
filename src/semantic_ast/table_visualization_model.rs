//! Non-executing table visualization and radio-table DTOs.

use super::{ParsedAnnotation, SectionIndexSource, TableColumnAlignment};

/// Org table visualization metadata collected without rendering plots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableVisualizationPlan<A = ()> {
    pub ann: A,
    pub source: Option<SectionIndexSource>,
    pub table_index: usize,
    pub kind: TableVisualizationKind,
    pub row_count: usize,
    pub column_count: usize,
    pub header: Vec<String>,
    pub column_alignments: Vec<Option<TableColumnAlignment>>,
    pub plot: Option<TablePlot<A>>,
    pub radio: Option<RadioTable<A>>,
    pub warnings: Vec<TableVisualizationWarning>,
}

/// Stable table source kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableVisualizationKind {
    OrgTable,
    TableEl,
}

impl TableVisualizationKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OrgTable => "orgTable",
            Self::TableEl => "tableEl",
        }
    }
}

/// Parsed `#+PLOT:` intent attached to a table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TablePlot<A = ()> {
    pub ann: A,
    pub raw: String,
    pub options: Vec<TableVisualizationOption>,
    pub title: Option<String>,
    pub plot_type: Option<TablePlotType>,
    pub with: Option<String>,
    pub file: Option<String>,
    pub index_column: Option<usize>,
    pub time_index_column: Option<usize>,
    pub dependent_columns: Vec<usize>,
    pub transpose: Option<bool>,
}

/// Org Plot type name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TablePlotType(String);

impl TablePlotType {
    /// Creates a plot type wrapper from source text.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the plot type name.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consumes the wrapper and returns the plot type name.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Parsed `#+ORGTBL: SEND ...` intent attached to a source table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioTable<A = ()> {
    pub ann: A,
    pub raw: String,
    pub name: String,
    pub translator: Option<String>,
    pub parameters: Vec<TableVisualizationOption>,
    pub receiver: Option<RadioTableReceiver>,
}

/// RECEIVE marker evidence for a radio table target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioTableReceiver {
    pub name: String,
    pub begin_found: bool,
    pub end_found: bool,
}

/// One normalized plot or radio-table option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableVisualizationOption {
    pub kind: TableVisualizationOptionKind,
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
}

/// Stable option category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableVisualizationOptionKind {
    Title,
    IndexColumn,
    TimeIndexColumn,
    DependentColumns,
    Transpose,
    Type,
    With,
    File,
    Set,
    Min,
    Max,
    Skip,
    SkipColumns,
    Splice,
    Format,
    Other,
}

impl TableVisualizationOptionKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::IndexColumn => "indexColumn",
            Self::TimeIndexColumn => "timeIndexColumn",
            Self::DependentColumns => "dependentColumns",
            Self::Transpose => "transpose",
            Self::Type => "type",
            Self::With => "with",
            Self::File => "file",
            Self::Set => "set",
            Self::Min => "min",
            Self::Max => "max",
            Self::Skip => "skip",
            Self::SkipColumns => "skipColumns",
            Self::Splice => "splice",
            Self::Format => "format",
            Self::Other => "other",
        }
    }
}

/// Non-fatal table visualization diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableVisualizationWarning {
    pub kind: TableVisualizationWarningKind,
    pub message: String,
}

/// Stable visualization warning category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableVisualizationWarningKind {
    InvalidPlotOption,
    InvalidRadioTableDirective,
    MissingRadioReceiver,
}

impl TableVisualizationWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPlotOption => "invalidPlotOption",
            Self::InvalidRadioTableDirective => "invalidRadioTableDirective",
            Self::MissingRadioReceiver => "missingRadioReceiver",
        }
    }
}

impl TableVisualizationPlan<ParsedAnnotation> {
    /// Returns true when this plan carries any non-fatal warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
