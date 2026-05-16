//! Agenda view plans with sort and limit receipts for downstream consumers.

use super::{
    AgendaCategory, AgendaDate, AgendaEntryKind, AgendaQuery, AgendaTime, AgendaUrgencyScore,
    SectionIndexSource, TaskBlockerRecord, TodoKeyword,
};

/// Query wrapper for explainable agenda view plans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaViewQuery {
    pub agenda: AgendaQuery,
    pub limit: Option<usize>,
    pub sort_strategy: Vec<AgendaViewSortSpec>,
}

impl AgendaViewQuery {
    /// Creates a view query from an existing agenda query.
    pub fn new(agenda: AgendaQuery) -> Self {
        Self {
            agenda,
            limit: None,
            sort_strategy: Vec::new(),
        }
    }

    /// Creates a view query for one day.
    pub fn single_day(date: AgendaDate) -> Self {
        Self::new(AgendaQuery::single_day(date))
    }

    /// Limits the accepted cards while preserving skipped receipts.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Applies an Org agenda-style sort strategy subset.
    pub fn sort_strategy(mut self, strategy: impl IntoIterator<Item = AgendaViewSortSpec>) -> Self {
        self.sort_strategy = strategy.into_iter().collect();
        self
    }

    /// Appends one sort spec.
    pub fn sort_by(mut self, spec: AgendaViewSortSpec) -> Self {
        self.sort_strategy.push(spec);
        self
    }
}

/// One explainable document-local agenda view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaViewPlan {
    pub total_candidates: usize,
    pub limit: Option<usize>,
    pub sort_strategy: Vec<AgendaViewSortSpec>,
    pub cards: Vec<AgendaViewCard>,
    pub skipped: Vec<AgendaViewSkip>,
}

impl AgendaViewPlan {
    /// Returns true when no card was accepted into the view.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Renders a compact receipt-oriented agenda plan for agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        if self.cards.is_empty() && self.skipped.is_empty() {
            return "[ok] orgize agenda view\n".to_string();
        }

        let mut output = String::new();
        output.push_str("agenda-view: ");
        output.push_str(&self.cards.len().to_string());
        output.push_str(" accepted / ");
        output.push_str(&self.skipped.len().to_string());
        output.push_str(" skipped");
        if let Some(limit) = self.limit {
            output.push_str(" / limit ");
            output.push_str(&limit.to_string());
        }
        if !self.sort_strategy.is_empty() {
            output.push_str(" / sort ");
            output.push_str(&compact_sort_strategy(&self.sort_strategy));
        }
        output.push('\n');
        for card in &self.cards {
            card.push_compact_text(path, &mut output);
        }
        for skip in &self.skipped {
            skip.push_compact_text(path, &mut output);
        }
        output
    }
}

/// One accepted agenda card with audit receipts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaViewCard {
    pub source: SectionIndexSource,
    pub sorted_position: usize,
    pub kind: AgendaEntryKind,
    pub display_date: AgendaDate,
    pub target_date: AgendaDate,
    pub target_end_date: Option<AgendaDate>,
    pub time: Option<AgendaTime>,
    pub end_time: Option<AgendaTime>,
    pub title: String,
    pub category: Option<AgendaCategory>,
    pub todo: Option<TodoKeyword>,
    pub effective_tags: Vec<String>,
    pub urgency: AgendaUrgencyScore,
    pub blockers: Vec<TaskBlockerRecord>,
    pub sort_keys: Vec<AgendaViewSortValue>,
    pub receipts: Vec<AgendaViewReceipt>,
}

impl AgendaViewCard {
    fn push_compact_text(&self, path: &str, output: &mut String) {
        output.push_str("[AGENDA_ACCEPT] ");
        output.push_str(&self.title);
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        output.push_str("position: ");
        output.push_str(&self.sorted_position.to_string());
        output.push('\n');
        output.push_str("date: ");
        output.push_str(&format_date(self.display_date));
        output.push('\n');
        output.push_str("kind: ");
        output.push_str(self.kind.as_str());
        output.push('\n');
        output.push_str("receipt: ");
        output.push_str(
            &self
                .receipts
                .iter()
                .map(|receipt| receipt.kind.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
        for blocker in &self.blockers {
            output.push_str("blocked-by: ");
            output.push_str(blocker.kind.as_str());
            output.push(' ');
            output.push_str(&blocker.blocker.title);
            output.push_str(" @ ");
            output.push_str(&blocker.blocker.source.start.line.to_string());
            output.push(':');
            output.push_str(&blocker.blocker.source.start.column.to_string());
            output.push('\n');
        }
    }
}

/// One skipped agenda row with its reason preserved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaViewSkip {
    pub source: SectionIndexSource,
    pub sorted_position: usize,
    pub title: String,
    pub reason: AgendaViewSkipReason,
    pub urgency: AgendaUrgencyScore,
    pub blockers: Vec<TaskBlockerRecord>,
    pub sort_keys: Vec<AgendaViewSortValue>,
    pub receipts: Vec<AgendaViewReceipt>,
}

impl AgendaViewSkip {
    fn push_compact_text(&self, path: &str, output: &mut String) {
        output.push_str("[AGENDA_SKIP] ");
        output.push_str(&self.title);
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        output.push_str("position: ");
        output.push_str(&self.sorted_position.to_string());
        output.push('\n');
        output.push_str("reason: ");
        output.push_str(self.reason.as_str());
        output.push('\n');
        for blocker in &self.blockers {
            output.push_str("blocked-by: ");
            output.push_str(blocker.kind.as_str());
            output.push(' ');
            output.push_str(&blocker.blocker.title);
            output.push_str(" @ ");
            output.push_str(&blocker.blocker.source.start.line.to_string());
            output.push(':');
            output.push_str(&blocker.blocker.source.start.column.to_string());
            output.push('\n');
        }
    }
}

/// Why an agenda row was skipped from the accepted card list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgendaViewSkipReason {
    Limit { limit: usize },
}

impl AgendaViewSkipReason {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Limit { .. } => "limit",
        }
    }
}

/// One audit receipt for an agenda view decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaViewReceipt {
    pub kind: AgendaViewReceiptKind,
    pub message: String,
}

/// Stable receipt kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaViewReceiptKind {
    QueryMatched,
    Sorted,
    Accepted,
    BlockedByOrderedSibling,
    SkippedLimit,
}

impl AgendaViewReceiptKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QueryMatched => "queryMatched",
            Self::Sorted => "sorted",
            Self::Accepted => "accepted",
            Self::BlockedByOrderedSibling => "blockedByOrderedSibling",
            Self::SkippedLimit => "skippedLimit",
        }
    }
}

/// One sort key used by the default agenda view order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaViewSortValue {
    pub key: AgendaViewSortKey,
    pub value: String,
}

/// One agenda sort strategy selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgendaViewSortSpec {
    pub key: AgendaViewSortKey,
    pub direction: AgendaViewSortDirection,
}

impl AgendaViewSortSpec {
    /// Creates one sort spec.
    pub const fn new(key: AgendaViewSortKey, direction: AgendaViewSortDirection) -> Self {
        Self { key, direction }
    }

    /// Sorts a key in ascending Org-style order.
    pub const fn up(key: AgendaViewSortKey) -> Self {
        Self::new(key, AgendaViewSortDirection::Up)
    }

    /// Sorts a key in descending Org-style order.
    pub const fn down(key: AgendaViewSortKey) -> Self {
        Self::new(key, AgendaViewSortDirection::Down)
    }

    /// Keeps the incoming candidate order for this key.
    pub const fn keep(key: AgendaViewSortKey) -> Self {
        Self::new(key, AgendaViewSortDirection::Keep)
    }
}

/// Sort direction for an agenda strategy selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaViewSortDirection {
    Up,
    Down,
    Keep,
}

impl AgendaViewSortDirection {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Keep => "keep",
        }
    }
}

/// Stable sort-key labels for agenda order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaViewSortKey {
    DisplayDate,
    Time,
    Kind,
    Level,
    Title,
    TargetDate,
    ScheduledDate,
    DeadlineDate,
    Priority,
    Category,
    TodoState,
}

impl AgendaViewSortKey {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DisplayDate => "displayDate",
            Self::Time => "time",
            Self::Kind => "kind",
            Self::Level => "level",
            Self::Title => "title",
            Self::TargetDate => "targetDate",
            Self::ScheduledDate => "scheduledDate",
            Self::DeadlineDate => "deadlineDate",
            Self::Priority => "priority",
            Self::Category => "category",
            Self::TodoState => "todoState",
        }
    }
}

/// Multi-section block agenda query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaBlockViewQuery {
    pub title: String,
    pub sections: Vec<AgendaBlockSectionQuery>,
}

impl AgendaBlockViewQuery {
    /// Creates an empty block agenda query.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    /// Adds one block section.
    pub fn section(mut self, name: impl Into<String>, query: AgendaViewQuery) -> Self {
        self.sections.push(AgendaBlockSectionQuery {
            name: name.into(),
            query,
        });
        self
    }
}

/// One named block agenda section query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaBlockSectionQuery {
    pub name: String,
    pub query: AgendaViewQuery,
}

/// Multi-section block agenda plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaBlockViewPlan {
    pub title: String,
    pub total_candidates: usize,
    pub sections: Vec<AgendaBlockSectionPlan>,
}

impl AgendaBlockViewPlan {
    /// Renders a compact receipt-oriented block agenda plan for agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        let mut output = String::new();
        output.push_str("agenda-block: ");
        output.push_str(&self.title);
        output.push_str(" / ");
        output.push_str(&self.sections.len().to_string());
        output.push_str(" sections / ");
        output.push_str(&self.total_candidates.to_string());
        output.push_str(" candidates\n");
        for section in &self.sections {
            output.push_str("[AGENDA_SECTION] ");
            output.push_str(&section.index.to_string());
            output.push(' ');
            output.push_str(&section.name);
            output.push('\n');
            output.push_str(&section.plan.to_compact_text(path));
        }
        output
    }
}

/// One planned block agenda section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaBlockSectionPlan {
    pub index: usize,
    pub name: String,
    pub plan: AgendaViewPlan,
}

pub(crate) fn format_date(date: AgendaDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}

pub(crate) fn format_time(time: Option<AgendaTime>) -> String {
    match time {
        Some(time) => format!("{:02}:{:02}", time.hour, time.minute),
        None => "none".to_string(),
    }
}

pub(crate) fn compact_sort_strategy(strategy: &[AgendaViewSortSpec]) -> String {
    if strategy.is_empty() {
        return "default".to_string();
    }
    strategy
        .iter()
        .map(|spec| format!("{}-{}", spec.key.as_str(), spec.direction.as_str()))
        .collect::<Vec<_>>()
        .join(",")
}
