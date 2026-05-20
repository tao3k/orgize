//! Habit-oriented agenda metadata.

use super::TodoKeyword;
use super::property_model::OrgDuration;
use super::section_index_model::SectionIndexSource;
use super::timestamp_model::{Timestamp, TimestampRepeater};

/// One `STYLE: habit` headline projected for agenda/indexer consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HabitRecord {
    pub source: SectionIndexSource,
    pub level: usize,
    pub title: String,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub scheduled: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
    pub repeater: Option<TimestampRepeater>,
    pub last_repeat: Option<HabitLastRepeat>,
    pub effort: Option<OrgDuration>,
    pub clock_count: usize,
    pub clock_total_seconds: u64,
    pub consistency: HabitConsistency,
}

/// `LAST_REPEAT` property evidence for a habit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HabitLastRepeat {
    pub source: SectionIndexSource,
    pub raw: String,
}

/// Minimum structured inputs needed to render or lint habit consistency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HabitConsistency {
    /// Scheduled timestamp, repeater, and last repeat are all present.
    Complete,
    /// Habit has no `SCHEDULED` timestamp.
    MissingScheduled,
    /// Habit has no repeater on its scheduled/deadline timestamp.
    MissingRepeater,
    /// Habit has no `LAST_REPEAT` evidence yet.
    MissingLastRepeat,
}
