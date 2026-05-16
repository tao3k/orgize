//! Workspace-level agenda command plan DTOs.

use super::{
    AgendaMatchQuery, AgendaTime, AgendaUrgencyScore, AgendaViewQuery, SectionIndexSource,
    TodoKeyword,
};

/// Query containing named Org Agenda-style workspace commands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgendaWorkspaceQuery {
    pub commands: Vec<AgendaWorkspaceCommandQuery>,
}

impl AgendaWorkspaceQuery {
    /// Creates an empty workspace agenda query.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a named command.
    pub fn command(mut self, name: impl Into<String>, kind: AgendaWorkspaceCommandKind) -> Self {
        self.commands.push(AgendaWorkspaceCommandQuery {
            name: name.into(),
            kind,
        });
        self
    }
}

/// One named workspace agenda command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceCommandQuery {
    pub name: String,
    pub kind: AgendaWorkspaceCommandKind,
}

/// Supported non-executing workspace agenda command kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgendaWorkspaceCommandKind {
    Agenda(AgendaViewQuery),
    TodoList {
        include_done: bool,
    },
    Match(AgendaWorkspaceMatchCommand),
    Search {
        needle: String,
        case_sensitive: bool,
    },
    StuckProjects {
        next_keywords: Vec<String>,
    },
}

impl AgendaWorkspaceCommandKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(&self) -> &'static str {
        self.label().as_str()
    }

    /// Stable typed label for this command kind.
    pub const fn label(&self) -> AgendaWorkspaceCommandKindLabel {
        match self {
            Self::Agenda(_) => AgendaWorkspaceCommandKindLabel::Agenda,
            Self::TodoList { .. } => AgendaWorkspaceCommandKindLabel::TodoList,
            Self::Match(_) => AgendaWorkspaceCommandKindLabel::Match,
            Self::Search { .. } => AgendaWorkspaceCommandKindLabel::Search,
            Self::StuckProjects { .. } => AgendaWorkspaceCommandKindLabel::StuckProjects,
        }
    }
}

/// Match command options as a named public payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceMatchCommand {
    pub query: AgendaMatchQuery,
    pub include_done: bool,
    pub include_archived: bool,
}

impl AgendaWorkspaceMatchCommand {
    /// Creates a match command with explicit visibility switches.
    pub fn new(query: AgendaMatchQuery) -> Self {
        Self {
            query,
            include_done: false,
            include_archived: false,
        }
    }

    /// Includes DONE headings in match output.
    pub const fn include_done(mut self, include_done: bool) -> Self {
        self.include_done = include_done;
        self
    }

    /// Includes archived headings in match output.
    pub const fn include_archived(mut self, include_archived: bool) -> Self {
        self.include_archived = include_archived;
        self
    }
}

/// Stable workspace command category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaWorkspaceCommandKindLabel {
    Agenda,
    TodoList,
    Match,
    Search,
    StuckProjects,
}

impl AgendaWorkspaceCommandKindLabel {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Agenda => "agenda",
            Self::TodoList => "todoList",
            Self::Match => "match",
            Self::Search => "search",
            Self::StuckProjects => "stuckProjects",
        }
    }
}

impl std::fmt::Display for AgendaWorkspaceCommandKindLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Result of running named agenda commands over caller-supplied documents.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgendaWorkspacePlan {
    pub documents: Vec<AgendaWorkspaceDocumentSummary>,
    pub commands: Vec<AgendaWorkspaceCommandPlan>,
}

/// Input document summary preserved in a workspace plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceDocumentSummary {
    pub source_file: String,
    pub section_count: usize,
}

/// One named command result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceCommandPlan {
    pub name: String,
    pub kind: AgendaWorkspaceCommandKindLabel,
    pub total_candidates: usize,
    pub cards: Vec<AgendaWorkspaceCard>,
    pub skipped: Vec<AgendaWorkspaceSkip>,
}

/// One accepted workspace agenda/search/TODO card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceCard {
    pub source_file: String,
    pub source: SectionIndexSource,
    pub title: String,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub kind: AgendaWorkspaceCardKind,
    pub todo: Option<TodoKeyword>,
    pub time: Option<AgendaTime>,
    pub urgency: AgendaUrgencyScore,
    pub receipts: Vec<AgendaWorkspaceReceipt>,
}

/// One skipped workspace agenda card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceSkip {
    pub source_file: String,
    pub source: SectionIndexSource,
    pub title: String,
    pub reason: AgendaWorkspaceSkipReason,
    pub receipts: Vec<AgendaWorkspaceReceipt>,
}

/// Stable workspace card category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaWorkspaceCardKind {
    Agenda,
    Todo,
    Match,
    Search,
    StuckProject,
}

impl AgendaWorkspaceCardKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Agenda => "agenda",
            Self::Todo => "todo",
            Self::Match => "match",
            Self::Search => "search",
            Self::StuckProject => "stuckProject",
        }
    }
}

/// Stable workspace skip reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgendaWorkspaceSkipReason {
    AgendaViewLimit,
}

impl AgendaWorkspaceSkipReason {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AgendaViewLimit => "agendaViewLimit",
        }
    }
}

/// One receipt explaining why a workspace card exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgendaWorkspaceReceipt {
    pub kind: AgendaWorkspaceReceiptKind,
    pub message: String,
}

/// Stable workspace receipt category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgendaWorkspaceReceiptKind {
    DocumentAccepted,
    QueryMatched,
    SearchMatched,
    TodoMatched,
    StuckProjectMatched,
    AgendaViewSkipped,
}

impl AgendaWorkspaceReceiptKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentAccepted => "documentAccepted",
            Self::QueryMatched => "queryMatched",
            Self::SearchMatched => "searchMatched",
            Self::TodoMatched => "todoMatched",
            Self::StuckProjectMatched => "stuckProjectMatched",
            Self::AgendaViewSkipped => "agendaViewSkipped",
        }
    }
}
