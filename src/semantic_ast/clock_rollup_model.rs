//! Clock rollups and clocktable plans for agent-facing time evidence.

use super::{OrgDuration, SectionIndexSource};

/// One source-grounded clock and effort rollup for an Org section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockRollupRecord {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub local_clock: ClockSummary,
    pub subtree_clock: ClockSummary,
    pub effort: ClockEffortSummary,
}

/// Parsed CLOCK summary for a local section body or subtree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockSummary {
    pub entries: usize,
    pub closed_entries: usize,
    pub running_entries: usize,
    pub unparsed_entries: usize,
    pub total_seconds: u64,
}

impl ClockSummary {
    pub(crate) fn merge(&mut self, other: Self) {
        self.entries += other.entries;
        self.closed_entries += other.closed_entries;
        self.running_entries += other.running_entries;
        self.unparsed_entries += other.unparsed_entries;
        self.total_seconds += other.total_seconds;
    }
}

/// Effort comparison derived from native `EFFORT` properties and CLOCK lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockEffortSummary {
    pub local: Option<OrgDuration>,
    pub subtree_total_seconds: u64,
    pub delta_seconds: i64,
    pub status: ClockEffortStatus,
}

/// Clock-vs-effort comparison status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockEffortStatus {
    NoEffort,
    UnderEffort,
    OnEffort,
    OverEffort,
}

impl ClockEffortStatus {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoEffort => "noEffort",
            Self::UnderEffort => "underEffort",
            Self::OnEffort => "onEffort",
            Self::OverEffort => "overEffort",
        }
    }
}

/// Non-mutating plan for one `#+BEGIN: clocktable` dynamic block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTablePlan {
    pub source: SectionIndexSource,
    pub name: String,
    pub parameters: Vec<ClockTableParameter>,
    pub scope: ClockTableScope,
    pub max_level: usize,
    pub tstart: Option<String>,
    pub tend: Option<String>,
    pub time_window: Option<ClockTableTimeWindow>,
    pub match_filter: Option<ClockTableMatchFilter>,
    pub property_columns: Option<ClockTablePropertyColumns>,
    pub rows: Vec<ClockTableRow>,
    pub warnings: Vec<ClockTableWarning>,
}

impl ClockTablePlan {
    /// Renders clocktable rows as compact text for coding agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        let mut output = String::new();
        self.push_compact_header(path, &mut output);
        self.push_compact_parameters(&mut output);
        self.push_compact_rows(&mut output);
        self.push_compact_warnings(&mut output);
        output.push_str("contract: Derived from official Org CLOCK, EFFORT, and clocktable dynamic-block constructs; no custom source syntax is required.");
        output.push('\n');
        output
    }

    fn push_compact_header(&self, path: &str, output: &mut String) {
        output.push_str("[CLOCKTABLE] ");
        output.push_str(&self.name);
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        output.push_str("scope: ");
        output.push_str(self.scope.kind.as_str());
        if let Some(value) = &self.scope.value {
            output.push('(');
            output.push_str(value);
            output.push(')');
        }
        output.push_str(" maxlevel: ");
        output.push_str(&self.max_level.to_string());
        output.push('\n');
        if let Some(time_window) = &self.time_window {
            output.push_str("window: ");
            output.push_str(time_window.source.as_str());
            output.push_str(" [");
            if let Some(start) = &time_window.start {
                start.push_compact_text(output);
            } else {
                output.push_str("-inf");
            }
            output.push_str(", ");
            if let Some(end_exclusive) = &time_window.end_exclusive {
                end_exclusive.push_compact_text(output);
            } else {
                output.push_str("+inf");
            }
            output.push_str(")\n");
        }
        if let Some(match_filter) = &self.match_filter {
            output.push_str("match: ");
            output.push_str(&match_filter.expression);
            output.push('\n');
        }
        if let Some(property_columns) = &self.property_columns {
            output.push_str("properties: ");
            output.push_str(&property_columns.names.join(", "));
            output.push_str(" inherit=");
            output.push_str(if property_columns.inherit {
                "true"
            } else {
                "false"
            });
            output.push('\n');
        }
    }

    fn push_compact_parameters(&self, output: &mut String) {
        if !self.parameters.is_empty() {
            output.push_str("params: ");
            output.push_str(
                &self
                    .parameters
                    .iter()
                    .map(|parameter| parameter.raw.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            output.push('\n');
        }
    }

    fn push_compact_rows(&self, output: &mut String) {
        for row in &self.rows {
            output.push_str("row: ");
            output.push_str(&row.outline_path.join(" / "));
            output.push_str(" | tableLevel=");
            output.push_str(&row.table_level.to_string());
            output.push_str(" | clock=");
            output.push_str(&row.clock.total_seconds.to_string());
            output.push_str("s/");
            output.push_str(&row.clock.entries.to_string());
            output.push_str(" entries | effort=");
            output.push_str(&row.effort_total_seconds.to_string());
            output.push_str("s | delta=");
            output.push_str(&row.effort_delta_seconds.to_string());
            output.push_str("s | status=");
            output.push_str(row.effort_status.as_str());
            output.push('\n');
        }
    }

    fn push_compact_warnings(&self, output: &mut String) {
        for warning in &self.warnings {
            output.push_str("warning: ");
            output.push_str(warning.kind.as_str());
            output.push_str(" - ");
            output.push_str(&warning.message);
            output.push('\n');
        }
    }
}

/// One native clocktable parameter from the dynamic-block begin line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTableParameter {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
}

/// Projected clocktable scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTableScope {
    pub kind: ClockTableScopeKind,
    pub value: Option<String>,
}

/// Static wall-clock window applied to clocktable row totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTableTimeWindow {
    pub source: ClockTableTimeWindowSource,
    pub start: Option<ClockTableTimeBound>,
    pub end_exclusive: Option<ClockTableTimeBound>,
}

/// Origin of a clocktable time window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockTableTimeWindowSource {
    Block,
    TstartTend,
}

impl ClockTableTimeWindowSource {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::TstartTend => "tstartTend",
        }
    }
}

/// Date-time bound used by a clocktable window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTableTimeBound {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl ClockTableTimeBound {
    fn push_compact_text(&self, output: &mut String) {
        use std::fmt::Write as _;

        let _ = write!(
            output,
            "{:04}-{:02}-{:02}T{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        );
    }
}

/// Parsed `:match` filter applied to clocktable clock and Effort totals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTableMatchFilter {
    pub expression: String,
}

/// Property columns requested by native clocktable `:properties`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTablePropertyColumns {
    pub names: Vec<String>,
    pub inherit: bool,
}

/// Supported and preserved Org clocktable scope forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockTableScopeKind {
    File,
    Subtree,
    Tree,
    TreeLevel,
    Agenda,
    AgendaWithArchives,
    FileWithArchives,
    Nil,
    External,
    Unknown,
}

impl ClockTableScopeKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Subtree => "subtree",
            Self::Tree => "tree",
            Self::TreeLevel => "treeLevel",
            Self::Agenda => "agenda",
            Self::AgendaWithArchives => "agendaWithArchives",
            Self::FileWithArchives => "fileWithArchives",
            Self::Nil => "nil",
            Self::External => "external",
            Self::Unknown => "unknown",
        }
    }
}

/// One row the clocktable plan would expose to downstream consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTableRow {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub table_level: usize,
    pub title: String,
    pub clock: ClockSummary,
    pub effort_total_seconds: u64,
    pub effort_delta_seconds: i64,
    pub effort_status: ClockEffortStatus,
    pub property_values: Vec<ClockTablePropertyValue>,
}

/// One clocktable property-column value for a projected row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTablePropertyValue {
    pub name: String,
    pub value: Option<String>,
    pub inherited: bool,
}

/// Non-fatal clocktable plan warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockTableWarning {
    pub kind: ClockTableWarningKind,
    pub message: String,
}

/// Stable warning kind for clocktable plans.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockTableWarningKind {
    UnsupportedScope,
    TimeRangePreserved,
    BlockRangePreserved,
    MatchPreserved,
    PropertiesPreserved,
    StepPreserved,
}

impl ClockTableWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedScope => "unsupportedScope",
            Self::TimeRangePreserved => "timeRangePreserved",
            Self::BlockRangePreserved => "blockRangePreserved",
            Self::MatchPreserved => "matchPreserved",
            Self::PropertiesPreserved => "propertiesPreserved",
            Self::StepPreserved => "stepPreserved",
        }
    }
}
