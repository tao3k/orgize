//! Timestamp semantic data model.

/// Timestamp metadata projected from Org timestamp syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub kind: TimestampKind,
    pub raw: String,
    pub is_range: bool,
    pub start: Option<TimestampMoment>,
    pub end: Option<TimestampMoment>,
    pub repeater: Option<TimestampRepeater>,
    pub warning: Option<TimestampWarning>,
}

/// Org timestamp delimiter category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimestampKind {
    /// Active timestamp, for example `<2026-05-01 Fri>`.
    Active,
    /// Inactive timestamp, for example `[2026-05-01 Fri]`.
    Inactive,
    /// Diary sexp timestamp, for example `<%%(diary-date 5 1)>`.
    Diary,
}

/// Parsed date and optional time within a timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampMoment {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub day_name: Option<String>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
}

/// Repeater cookie attached to a timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampRepeater {
    pub kind: RepeaterKind,
    pub value: u32,
    pub unit: TimeUnit,
}

/// Warning delay cookie attached to a timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampWarning {
    pub kind: WarningKind,
    pub value: u32,
    pub unit: TimeUnit,
}

/// Org timestamp repeater mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeaterKind {
    /// Cumulate repeater, written with `++`.
    Cumulate,
    /// Catch-up repeater, written with `+`.
    CatchUp,
    /// Restart repeater, written with `.+`.
    Restart,
}

/// Org timestamp warning delay mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarningKind {
    /// Warn for all matching occurrences.
    All,
    /// Warn only for the first occurrence.
    First,
}

/// Unit used by timestamp repeater and warning cookies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeUnit {
    /// Hour unit.
    Hour,
    /// Day unit.
    Day,
    /// Week unit.
    Week,
    /// Month unit.
    Month,
    /// Year unit.
    Year,
}
