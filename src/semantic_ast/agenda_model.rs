//! Agenda-oriented semantic projection types.

use super::model::{
    Object, TimeUnit, Timestamp, TimestampMoment, TimestampWarning, TodoKeyword, TodoState,
    WarningKind,
};

/// Inclusive date window and filters for semantic agenda projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaQuery {
    pub start: AgendaDate,
    pub end: AgendaDate,
    pub(crate) include_done: bool,
    pub(crate) include_comments: bool,
    pub(crate) include_archived: bool,
    pub(crate) include_scheduled: bool,
    pub(crate) include_deadlines: bool,
    pub(crate) include_timestamps: bool,
    pub(crate) include_closed: bool,
    pub(crate) expand_repeaters: bool,
    pub(crate) include_deadline_warnings: bool,
    pub(crate) include_overdue_deadlines: bool,
    pub(crate) search_headline_time: bool,
    pub(crate) required_tags: Vec<String>,
    pub(crate) excluded_tags: Vec<String>,
}

impl AgendaQuery {
    /// Creates an agenda query over an inclusive date range.
    pub fn new(start: AgendaDate, end: AgendaDate) -> Self {
        Self {
            start,
            end,
            include_done: false,
            include_comments: false,
            include_archived: false,
            include_scheduled: true,
            include_deadlines: true,
            include_timestamps: true,
            include_closed: false,
            expand_repeaters: true,
            include_deadline_warnings: true,
            include_overdue_deadlines: true,
            search_headline_time: true,
            required_tags: Vec::new(),
            excluded_tags: Vec::new(),
        }
    }

    /// Creates an agenda query for a single day.
    pub fn single_day(date: AgendaDate) -> Self {
        Self::new(date, date)
    }

    /// Includes or excludes headlines with DONE-state TODO keywords.
    pub fn include_done(mut self, include_done: bool) -> Self {
        self.include_done = include_done;
        self
    }

    /// Includes or excludes COMMENT headlines.
    pub fn include_comments(mut self, include_comments: bool) -> Self {
        self.include_comments = include_comments;
        self
    }

    /// Includes or excludes headlines inheriting the `ARCHIVE` tag.
    pub fn include_archived(mut self, include_archived: bool) -> Self {
        self.include_archived = include_archived;
        self
    }

    /// Includes or excludes `SCHEDULED` planning timestamps.
    pub fn include_scheduled(mut self, include_scheduled: bool) -> Self {
        self.include_scheduled = include_scheduled;
        self
    }

    /// Includes or excludes `DEADLINE` planning timestamps.
    pub fn include_deadlines(mut self, include_deadlines: bool) -> Self {
        self.include_deadlines = include_deadlines;
        self
    }

    /// Includes or excludes plain active timestamp objects.
    pub fn include_timestamps(mut self, include_timestamps: bool) -> Self {
        self.include_timestamps = include_timestamps;
        self
    }

    /// Includes or excludes `CLOSED` planning timestamps.
    pub fn include_closed(mut self, include_closed: bool) -> Self {
        self.include_closed = include_closed;
        self
    }

    /// Enables or disables date-changing repeater expansion.
    pub fn expand_repeaters(mut self, expand_repeaters: bool) -> Self {
        self.expand_repeaters = expand_repeaters;
        self
    }

    /// Enables or disables deadline warning rows.
    pub fn include_deadline_warnings(mut self, include_deadline_warnings: bool) -> Self {
        self.include_deadline_warnings = include_deadline_warnings;
        self
    }

    /// Enables or disables overdue deadline rows on the query start date.
    pub fn include_overdue_deadlines(mut self, include_overdue_deadlines: bool) -> Self {
        self.include_overdue_deadlines = include_overdue_deadlines;
        self
    }

    /// Enables or disables plain time-of-day extraction from headline text.
    pub fn search_headline_time(mut self, search_headline_time: bool) -> Self {
        self.search_headline_time = search_headline_time;
        self
    }

    /// Requires a tag to be present in a section's effective tag set.
    pub fn require_tag(mut self, tag: impl Into<String>) -> Self {
        self.required_tags.push(tag.into());
        self
    }

    /// Excludes sections with a tag in their effective tag set.
    pub fn exclude_tag(mut self, tag: impl Into<String>) -> Self {
        self.excluded_tags.push(tag.into());
        self
    }

    pub(crate) fn bounds(&self) -> (AgendaDate, AgendaDate) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}

/// Calendar date used by semantic agenda projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgendaDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl AgendaDate {
    /// Creates a date without normalizing invalid calendar values.
    pub const fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub(crate) fn from_moment(moment: &TimestampMoment) -> Self {
        Self::new(moment.year, moment.month, moment.day)
    }

    pub(crate) fn add_days(self, days: i32) -> Self {
        Self::from_day_number(self.day_number() + days)
    }

    pub(crate) fn add_interval(self, value: i32, unit: TimeUnit) -> Option<Self> {
        match unit {
            TimeUnit::Hour => None,
            TimeUnit::Day => Some(self.add_days(value)),
            TimeUnit::Week => Some(self.add_days(value.saturating_mul(7))),
            TimeUnit::Month => Some(self.add_months(value)),
            TimeUnit::Year => Some(self.add_months(value.saturating_mul(12))),
        }
    }

    pub(crate) fn days_until(self, other: Self) -> i32 {
        other.day_number() - self.day_number()
    }

    fn add_months(self, months: i32) -> Self {
        let total = i32::from(self.year) * 12 + i32::from(self.month) - 1 + months;
        let year = total.div_euclid(12).clamp(1, i32::from(u16::MAX));
        let month = total.rem_euclid(12) + 1;
        let day = self.day.min(days_in_month(year, month));
        Self::new(year as u16, month as u8, day)
    }

    fn day_number(self) -> i32 {
        days_from_civil(
            i32::from(self.year),
            u32::from(self.month),
            u32::from(self.day),
        )
    }

    fn from_day_number(day_number: i32) -> Self {
        let (year, month, day) = civil_from_days(day_number);
        Self::new(
            year.clamp(1, i32::from(u16::MAX)) as u16,
            month as u8,
            day as u8,
        )
    }
}

/// Time of day projected for an agenda row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgendaTime {
    pub hour: u8,
    pub minute: u8,
}

impl AgendaTime {
    pub(crate) fn from_moment(moment: &TimestampMoment) -> Option<Self> {
        Some(Self {
            hour: moment.hour?,
            minute: moment.minute.unwrap_or(0),
        })
    }
}

/// Category displayed for an agenda entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaCategory(String);

impl AgendaCategory {
    /// Creates an agenda category from parser-owned text.
    pub fn new(category: impl Into<String>) -> Self {
        Self(category.into())
    }

    /// Returns the category text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the category and returns the category text.
    pub fn into_string(self) -> String {
        self.0
    }
}

/// One semantic agenda row derived from a planning timestamp.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaEntry<A = ()> {
    pub ann: A,
    pub kind: AgendaEntryKind,
    pub display_date: AgendaDate,
    pub target_date: AgendaDate,
    pub target_end_date: Option<AgendaDate>,
    pub time: Option<AgendaTime>,
    pub end_time: Option<AgendaTime>,
    pub title: Vec<Object<A>>,
    pub raw_title: String,
    pub category: Option<AgendaCategory>,
    pub level: usize,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub anchor: Option<String>,
    pub timestamp: Timestamp,
    pub occurrence: AgendaOccurrence,
    pub scheduled: Option<AgendaScheduleState>,
    pub deadline: Option<AgendaDeadlineState>,
}

/// Planning timestamp category represented in agenda projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaEntryKind {
    Scheduled,
    Deadline,
    Timestamp,
    Closed,
}

/// Whether an agenda row is the source timestamp or a repeated occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaOccurrence {
    Source,
    Repeater { index: u32 },
}

/// Display state for a deadline agenda row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaDeadlineState {
    Due,
    Warning { days_until: u32 },
    Overdue { days_overdue: u32 },
}

/// Display state for a scheduled agenda row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaScheduleState {
    OnDate,
    Delayed { days_delayed: u32 },
    PastDue { days_overdue: u32 },
}

pub(crate) fn is_done_keyword(todo: &Option<TodoKeyword>) -> bool {
    matches!(todo.as_ref().map(|todo| todo.state), Some(TodoState::Done))
}

pub(crate) fn warning_start(date: AgendaDate, warning: Option<&TimestampWarning>) -> AgendaDate {
    warning
        .and_then(|warning| date.add_interval(-(warning.value as i32), warning.unit))
        .unwrap_or(date)
}

pub(crate) fn scheduled_visible_start(
    date: AgendaDate,
    warning: Option<&TimestampWarning>,
    occurrence: AgendaOccurrence,
) -> AgendaDate {
    let Some(warning) = warning else {
        return date;
    };
    let applies = warning.kind == WarningKind::All || occurrence == AgendaOccurrence::Source;
    if applies {
        date.add_interval(warning.value as i32, warning.unit)
            .unwrap_or(date)
    } else {
        date
    }
}

fn days_in_month(year: i32, month: i32) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 28,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(day_number: i32) -> (i32, u32, u32) {
    let z = day_number + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let month_prime = (5 * doy + 2) / 153;
    let day = doy - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i32::from(month <= 2);
    (year, month as u32, day as u32)
}
