//! Clock consistency diagnostics for agent-facing time evidence.

use super::{ClockTableTimeBound, SectionIndexSource};

/// Options for native Org clock consistency diagnostics.
///
/// The default profile mirrors `org-agenda-clock-consistency-checks`:
/// max duration 10:00, min duration 0, max gap 0:05, and gap-ok-around 4:00.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockIssueProfile {
    pub max_duration: Option<ClockIssueDurationThreshold>,
    pub min_duration: Option<ClockIssueDurationThreshold>,
    pub max_gap: Option<ClockIssueDurationThreshold>,
    pub gap_ok_around_minutes: Vec<u16>,
}

impl ClockIssueProfile {
    /// Org agenda default clock consistency profile.
    pub fn org_default() -> Self {
        Self {
            max_duration: Some(ClockIssueDurationThreshold::seconds(10 * 60 * 60)),
            min_duration: Some(ClockIssueDurationThreshold::seconds(0)),
            max_gap: Some(ClockIssueDurationThreshold::seconds(5 * 60)),
            gap_ok_around_minutes: vec![4 * 60],
        }
    }

    /// Set the maximum accepted closed clock duration.
    pub fn max_duration_seconds(mut self, seconds: u64) -> Self {
        self.max_duration = Some(ClockIssueDurationThreshold::seconds(seconds));
        self
    }

    /// Disable maximum duration diagnostics.
    pub fn without_max_duration(mut self) -> Self {
        self.max_duration = None;
        self
    }

    /// Set the minimum accepted closed clock duration.
    pub fn min_duration_seconds(mut self, seconds: u64) -> Self {
        self.min_duration = Some(ClockIssueDurationThreshold::seconds(seconds));
        self
    }

    /// Disable minimum duration diagnostics.
    pub fn without_min_duration(mut self) -> Self {
        self.min_duration = None;
        self
    }

    /// Set the maximum accepted gap between adjacent closed clocks.
    pub fn max_gap_seconds(mut self, seconds: u64) -> Self {
        self.max_gap = Some(ClockIssueDurationThreshold::seconds(seconds));
        self
    }

    /// Disable gap diagnostics.
    pub fn without_max_gap(mut self) -> Self {
        self.max_gap = None;
        self
    }

    /// Set time-of-day minutes that suppress gap diagnostics when contained in the gap.
    pub fn gap_ok_around_minutes(mut self, minutes: Vec<u16>) -> Self {
        self.gap_ok_around_minutes = minutes
            .into_iter()
            .filter(|minute| *minute < 24 * 60)
            .collect();
        self
    }
}

/// Duration threshold used by clock issue profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockIssueDurationThreshold {
    seconds: u64,
}

impl ClockIssueDurationThreshold {
    /// Build a threshold from seconds.
    pub const fn seconds(seconds: u64) -> Self {
        Self { seconds }
    }

    /// Return the threshold as seconds for DTO and comparison code.
    pub const fn as_seconds(self) -> u64 {
        self.seconds
    }
}

impl Default for ClockIssueProfile {
    fn default() -> Self {
        Self::org_default()
    }
}

/// One source-grounded clock consistency diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockIssueFinding {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub kind: ClockIssueFindingKind,
    pub message: String,
    pub clock: ClockIssueClock,
    pub previous_clock: Option<ClockIssueClock>,
    pub duration_seconds: Option<u64>,
    pub threshold_seconds: Option<u64>,
}

impl ClockIssueFinding {
    /// Render one finding as compact text for coding agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        let mut output = String::new();
        self.push_compact_header(path, &mut output);
        self.push_compact_clock_context(&mut output);
        self.push_compact_metrics(&mut output);
        self.push_compact_contract(&mut output);
        output
    }

    fn push_compact_header(&self, path: &str, output: &mut String) {
        output.push_str("[CLOCK-ISSUE] ");
        output.push_str(self.kind.as_str());
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        if !self.outline_path.is_empty() {
            output.push_str("outline: ");
            output.push_str(&self.outline_path.join(" / "));
            output.push('\n');
        }
    }

    fn push_compact_clock_context(&self, output: &mut String) {
        output.push_str("clock: ");
        output.push_str(self.clock.raw.trim());
        output.push('\n');
        if let Some(previous_clock) = &self.previous_clock {
            output.push_str("previous: ");
            output.push_str(previous_clock.raw.trim());
            output.push('\n');
        }
    }

    fn push_compact_metrics(&self, output: &mut String) {
        if let Some(duration_seconds) = self.duration_seconds {
            output.push_str("duration: ");
            output.push_str(&duration_seconds.to_string());
            output.push_str("s\n");
        }
        if let Some(threshold_seconds) = self.threshold_seconds {
            output.push_str("threshold: ");
            output.push_str(&threshold_seconds.to_string());
            output.push_str("s\n");
        }
        output.push_str("message: ");
        output.push_str(&self.message);
        output.push('\n');
    }

    fn push_compact_contract(&self, output: &mut String) {
        output.push_str("contract: Derived from official Org CLOCK agenda consistency checks; no custom source syntax is required.");
        output.push('\n');
    }
}

/// Stable clock issue kind for API and DTO consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockIssueFindingKind {
    InvalidClock,
    InvalidDuration,
    InvalidRange,
    NoEndTime,
    LongDuration,
    ShortDuration,
    Overlap,
    Gap,
}

impl ClockIssueFindingKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidClock => "invalidClock",
            Self::InvalidDuration => "invalidDuration",
            Self::InvalidRange => "invalidRange",
            Self::NoEndTime => "noEndTime",
            Self::LongDuration => "longDuration",
            Self::ShortDuration => "shortDuration",
            Self::Overlap => "overlap",
            Self::Gap => "gap",
        }
    }
}

/// Source and parsed interval data for a CLOCK line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockIssueClock {
    pub source: SectionIndexSource,
    pub raw: String,
    pub start: Option<ClockTableTimeBound>,
    pub end: Option<ClockTableTimeBound>,
    pub duration_seconds: Option<u64>,
}
