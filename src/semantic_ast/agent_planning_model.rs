//! Agent-facing planning snapshot types derived from Org agenda semantics.

use super::agenda_model::{
    AgendaCategory, AgendaDate, AgendaDeadlineState, AgendaEntry, AgendaEntryKind,
    AgendaOccurrence, AgendaQuery, AgendaScheduleState, AgendaTime,
};
use super::model::{ParsedAnnotation, SourcePosition, TodoKeyword};

/// Query wrapper for agent-facing planning snapshots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentPlanningQuery {
    pub agenda: AgendaQuery,
}

impl AgentPlanningQuery {
    /// Creates an agent planning query from the existing agenda query model.
    pub fn new(agenda: AgendaQuery) -> Self {
        Self { agenda }
    }

    /// Creates an agent planning query for a single day.
    pub fn single_day(date: AgendaDate) -> Self {
        Self::new(AgendaQuery::single_day(date))
    }
}

/// Compact planning snapshot intended for LLM-agent consumption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentPlanningSnapshot {
    pub cards: Vec<AgentPlanningCard>,
}

impl AgentPlanningSnapshot {
    /// Returns true when no planning card is visible for the query.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Renders planning cards as compact text for coding agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        if self.cards.is_empty() {
            return "[ok] orgize agent planning\n".to_string();
        }

        self.cards
            .iter()
            .map(|card| card.to_compact_text(path))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One compact decision card derived from an agenda row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentPlanningCard {
    pub source: AgentPlanningSource,
    pub decision: AgentPlanningDecision,
    pub display_date: AgendaDate,
    pub target_date: AgendaDate,
    pub target_end_date: Option<AgendaDate>,
    pub time: Option<AgendaTime>,
    pub end_time: Option<AgendaTime>,
    pub title: String,
    pub category: Option<AgendaCategory>,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub anchor: Option<String>,
    pub occurrence: AgendaOccurrence,
}

impl AgentPlanningCard {
    pub(crate) fn from_agenda_entry(entry: AgendaEntry<ParsedAnnotation>) -> Self {
        Self {
            source: AgentPlanningSource::from_annotation(&entry.ann),
            decision: AgentPlanningDecision::from_agenda_entry(&entry),
            display_date: entry.display_date,
            target_date: entry.target_date,
            target_end_date: entry.target_end_date,
            time: entry.time,
            end_time: entry.end_time,
            title: entry.raw_title,
            category: entry.category,
            todo: entry.todo,
            tags: entry.tags,
            effective_tags: entry.effective_tags,
            anchor: entry.anchor,
            occurrence: entry.occurrence,
        }
    }

    fn to_compact_text(&self, path: &str) -> String {
        let mut output = String::new();
        output.push('[');
        output.push_str(self.decision.code());
        output.push_str("] ");
        output.push_str(self.decision.severity().title());
        output.push_str(": ");
        output.push_str(self.decision.title());
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        output.push_str("date: ");
        output.push_str(&format_date(self.display_date));
        output.push('\n');
        output.push_str("task: ");
        output.push_str(&self.title);
        output.push('\n');
        if let Some(todo) = &self.todo {
            output.push_str("state: ");
            output.push_str(&todo.name);
            output.push('\n');
        }
        if let Some(category) = &self.category {
            output.push_str("category: ");
            output.push_str(category.as_str());
            output.push('\n');
        }
        if !self.effective_tags.is_empty() {
            output.push_str("tags: ");
            output.push_str(&self.effective_tags.join(":"));
            output.push('\n');
        }
        output.push_str("target: ");
        output.push_str(&self.decision.target_text(self));
        output.push('\n');
        output.push_str("next: ");
        output.push_str(self.decision.next_action());
        output.push('\n');
        output.push_str("contract: ");
        output.push_str(
            "Derived from official Org agenda syntax; no custom source syntax is required.",
        );
        output.push('\n');
        output
    }
}

/// Source location for an agent planning card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentPlanningSource {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: u32,
    pub range_end: u32,
}

impl AgentPlanningSource {
    fn from_annotation(annotation: &ParsedAnnotation) -> Self {
        Self {
            start: annotation.start,
            end: annotation.end,
            range_start: annotation.range.start().into(),
            range_end: annotation.range.end().into(),
        }
    }
}

/// Decision category for one agent planning card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentPlanningDecision {
    DeadlineDue,
    DeadlineWarning { days_until: u32 },
    DeadlineOverdue { days_overdue: u32 },
    Scheduled,
    ScheduledDelayed { days_delayed: u32 },
    ScheduledPastDue { days_overdue: u32 },
    ActiveTimestamp,
    Closed,
}

impl AgentPlanningDecision {
    fn from_agenda_entry(entry: &AgendaEntry<ParsedAnnotation>) -> Self {
        match entry.kind {
            AgendaEntryKind::Deadline => match entry.deadline {
                Some(AgendaDeadlineState::Warning { days_until }) => {
                    Self::DeadlineWarning { days_until }
                }
                Some(AgendaDeadlineState::Overdue { days_overdue }) => {
                    Self::DeadlineOverdue { days_overdue }
                }
                Some(AgendaDeadlineState::Due) | None => Self::DeadlineDue,
            },
            AgendaEntryKind::Scheduled => match entry.scheduled {
                Some(AgendaScheduleState::Delayed { days_delayed }) => {
                    Self::ScheduledDelayed { days_delayed }
                }
                Some(AgendaScheduleState::PastDue { days_overdue }) => {
                    Self::ScheduledPastDue { days_overdue }
                }
                Some(AgendaScheduleState::OnDate) | None => Self::Scheduled,
            },
            AgendaEntryKind::Timestamp => Self::ActiveTimestamp,
            AgendaEntryKind::Closed => Self::Closed,
        }
    }

    /// Stable compact output code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::DeadlineOverdue { .. } => "PLAN001",
            Self::DeadlineDue => "PLAN002",
            Self::DeadlineWarning { .. } => "PLAN003",
            Self::ScheduledPastDue { .. } => "PLAN004",
            Self::ScheduledDelayed { .. } => "PLAN005",
            Self::Scheduled => "PLAN006",
            Self::ActiveTimestamp => "PLAN007",
            Self::Closed => "PLAN008",
        }
    }

    /// Agent-facing severity derived from official agenda state.
    pub const fn severity(&self) -> AgentPlanningSeverity {
        match self {
            Self::DeadlineOverdue { .. } | Self::ScheduledPastDue { .. } => {
                AgentPlanningSeverity::Alert
            }
            Self::DeadlineDue
            | Self::DeadlineWarning { .. }
            | Self::ScheduledDelayed { .. }
            | Self::Scheduled => AgentPlanningSeverity::Action,
            Self::ActiveTimestamp | Self::Closed => AgentPlanningSeverity::Info,
        }
    }

    const fn title(&self) -> &'static str {
        match self {
            Self::DeadlineDue => "Deadline due",
            Self::DeadlineWarning { .. } => "Deadline warning",
            Self::DeadlineOverdue { .. } => "Deadline overdue",
            Self::Scheduled => "Scheduled task",
            Self::ScheduledDelayed { .. } => "Scheduled task delayed",
            Self::ScheduledPastDue { .. } => "Scheduled task past due",
            Self::ActiveTimestamp => "Active timestamp",
            Self::Closed => "Closed timestamp",
        }
    }

    fn target_text(&self, card: &AgentPlanningCard) -> String {
        match self {
            Self::DeadlineDue => {
                format!("deadline {}", format_date(card.target_date))
            }
            Self::DeadlineWarning { days_until } => {
                format!(
                    "deadline {}, due in {}",
                    format_date(card.target_date),
                    format_days(*days_until)
                )
            }
            Self::DeadlineOverdue { days_overdue } => {
                format!(
                    "deadline {}, overdue by {}",
                    format_date(card.target_date),
                    format_days(*days_overdue)
                )
            }
            Self::Scheduled => format!("scheduled {}", format_date(card.target_date)),
            Self::ScheduledDelayed { days_delayed } => {
                format!(
                    "scheduled {}, delayed by {}",
                    format_date(card.target_date),
                    format_days(*days_delayed)
                )
            }
            Self::ScheduledPastDue { days_overdue } => {
                format!(
                    "scheduled {}, past due by {}",
                    format_date(card.target_date),
                    format_days(*days_overdue)
                )
            }
            Self::ActiveTimestamp => {
                format!(
                    "active timestamp {}",
                    format_timed_date(card.target_date, card.time)
                )
            }
            Self::Closed => format!("closed {}", format_timed_date(card.target_date, card.time)),
        }
    }

    const fn next_action(&self) -> &'static str {
        match self {
            Self::DeadlineOverdue { .. } => {
                "finish, reschedule, or mark the Org task done if it is complete"
            }
            Self::DeadlineDue => "finish or explicitly reschedule the Org deadline",
            Self::DeadlineWarning { .. } => {
                "keep visible until the deadline is done or rescheduled"
            }
            Self::ScheduledPastDue { .. } => {
                "start, reschedule, or mark the Org task done if it is complete"
            }
            Self::ScheduledDelayed { .. } => {
                "show now because the official scheduled delay has elapsed"
            }
            Self::Scheduled => "start work when this scheduled date is visible",
            Self::ActiveTimestamp => "treat as a dated event or appointment, not a task start date",
            Self::Closed => {
                "treat as history unless the consumer explicitly includes closed entries"
            }
        }
    }
}

/// Agent-facing severity for planning cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentPlanningSeverity {
    Alert,
    Action,
    Info,
}

impl AgentPlanningSeverity {
    const fn title(self) -> &'static str {
        match self {
            Self::Alert => "Alert",
            Self::Action => "Action",
            Self::Info => "Info",
        }
    }
}

fn format_timed_date(date: AgendaDate, time: Option<AgendaTime>) -> String {
    match time {
        Some(time) => format!("{} {}", format_date(date), format_time(time)),
        None => format_date(date),
    }
}

fn format_date(date: AgendaDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}

fn format_time(time: AgendaTime) -> String {
    format!("{:02}:{:02}", time.hour, time.minute)
}

fn format_days(days: u32) -> String {
    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}
