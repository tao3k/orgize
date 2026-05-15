//! Agent-facing memory projection types derived from ordinary Org semantics.

use super::model::{ParsedAnnotation, SourcePosition, TimestampKind, TodoKeyword, TodoState};

/// Query for Org-native memory records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryQuery {
    pub(crate) include_comments: bool,
    pub(crate) include_closed: bool,
    pub(crate) include_archived: bool,
    pub(crate) required_tags: Vec<String>,
    pub(crate) excluded_tags: Vec<String>,
}

impl MemoryQuery {
    /// Creates a conservative memory query that keeps historical evidence
    /// visible instead of silently dropping closed or archived facts.
    pub fn new() -> Self {
        Self {
            include_comments: false,
            include_closed: true,
            include_archived: true,
            required_tags: Vec::new(),
            excluded_tags: Vec::new(),
        }
    }

    /// Includes or excludes COMMENT headlines.
    pub fn include_comments(mut self, include_comments: bool) -> Self {
        self.include_comments = include_comments;
        self
    }

    /// Includes or excludes DONE/CLOSED memory records.
    pub fn include_closed(mut self, include_closed: bool) -> Self {
        self.include_closed = include_closed;
        self
    }

    /// Includes or excludes records inheriting the `ARCHIVE` tag.
    pub fn include_archived(mut self, include_archived: bool) -> Self {
        self.include_archived = include_archived;
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
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Query wrapper for agent-facing memory snapshots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMemoryQuery {
    pub memory: MemoryQuery,
}

impl AgentMemoryQuery {
    /// Creates an agent memory query from the raw memory query model.
    pub fn new(memory: MemoryQuery) -> Self {
        Self { memory }
    }
}

/// One Org headline projected as an addressable memory record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryRecord {
    pub source: MemorySource,
    pub state: MemoryRecordState,
    pub level: usize,
    pub title: String,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub anchor: Option<String>,
    pub properties: Vec<MemoryProperty>,
    pub evidence: Vec<MemoryEvidence>,
    pub links: Vec<MemoryLink>,
}

/// Compact memory snapshot intended for LLM-agent consumption.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMemorySnapshot {
    pub cards: Vec<AgentMemoryCard>,
}

impl AgentMemorySnapshot {
    /// Returns true when no memory card is visible for the query.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Renders memory cards as compact text for coding agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        if self.cards.is_empty() {
            return "[ok] orgize agent memory\n".to_string();
        }

        self.cards
            .iter()
            .map(|card| card.to_compact_text(path))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One compact decision card derived from an Org memory record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentMemoryCard {
    pub source: MemorySource,
    pub decision: AgentMemoryDecision,
    pub title: String,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub anchor: Option<String>,
    pub evidence: Vec<MemoryEvidence>,
    pub links: Vec<MemoryLink>,
}

impl AgentMemoryCard {
    pub(crate) fn from_record(record: MemoryRecord) -> Self {
        Self {
            source: record.source,
            decision: AgentMemoryDecision::from_state(record.state),
            title: record.title,
            todo: record.todo,
            tags: record.tags,
            effective_tags: record.effective_tags,
            anchor: record.anchor,
            evidence: record.evidence,
            links: record.links,
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
        output.push_str("fact: ");
        output.push_str(&self.title);
        output.push('\n');
        if let Some(todo) = &self.todo {
            output.push_str("state: ");
            output.push_str(&todo.name);
            output.push('\n');
        }
        if !self.effective_tags.is_empty() {
            output.push_str("tags: ");
            output.push_str(&self.effective_tags.join(":"));
            output.push('\n');
        }
        let evidence = self
            .evidence
            .iter()
            .map(|item| item.kind.title())
            .collect::<Vec<_>>();
        if !evidence.is_empty() {
            output.push_str("evidence: ");
            output.push_str(&evidence.join(", "));
            output.push('\n');
        }
        if !self.links.is_empty() {
            output.push_str("links: ");
            output.push_str(
                &self
                    .links
                    .iter()
                    .map(|link| link.path.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            output.push('\n');
        }
        output.push_str("next: ");
        output.push_str(self.decision.next_action());
        output.push('\n');
        output.push_str("contract: ");
        output.push_str(
            "Derived from official Org memory-bearing constructs; no custom source syntax is required.",
        );
        output.push('\n');
        output
    }
}

/// Source location for memory records and memory evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySource {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: u32,
    pub range_end: u32,
}

impl MemorySource {
    pub(crate) fn from_annotation(annotation: &ParsedAnnotation) -> Self {
        Self {
            start: annotation.start,
            end: annotation.end,
            range_start: annotation.range.start().into(),
            range_end: annotation.range.end().into(),
        }
    }
}

/// Lifecycle state for one memory record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRecordState {
    Current,
    Closed,
    Archived,
    Background,
}

/// Property copied from a standard Org property drawer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryProperty {
    pub source: MemorySource,
    pub key: String,
    pub value: String,
}

/// Evidence that influenced a memory record or agent memory card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryEvidence {
    pub source: MemorySource,
    pub kind: MemoryEvidenceKind,
    pub value: String,
}

/// Kind of official Org evidence attached to a memory record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryEvidenceKind {
    TodoState,
    ArchiveTag,
    Property { key: String },
    Scheduled,
    Deadline,
    Closed,
    Timestamp { kind: TimestampKind },
    Logbook,
    Drawer { name: String },
    Clock,
    Link,
}

impl MemoryEvidenceKind {
    fn title(&self) -> String {
        match self {
            Self::TodoState => "TODO state".to_string(),
            Self::ArchiveTag => "ARCHIVE tag".to_string(),
            Self::Property { key } => format!("property {key}"),
            Self::Scheduled => "SCHEDULED".to_string(),
            Self::Deadline => "DEADLINE".to_string(),
            Self::Closed => "CLOSED".to_string(),
            Self::Timestamp { kind } => match kind {
                TimestampKind::Active => "active timestamp".to_string(),
                TimestampKind::Inactive => "inactive timestamp".to_string(),
                TimestampKind::Diary => "diary timestamp".to_string(),
            },
            Self::Logbook => "LOGBOOK".to_string(),
            Self::Drawer { name } => format!("drawer {name}"),
            Self::Clock => "CLOCK".to_string(),
            Self::Link => "link".to_string(),
        }
    }
}

/// Link evidence visible to a memory projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryLink {
    pub source: MemorySource,
    pub path: String,
    pub description: String,
}

/// Decision category for one agent memory card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentMemoryDecision {
    Current,
    Closed,
    Archived,
    Background,
}

impl AgentMemoryDecision {
    fn from_state(state: MemoryRecordState) -> Self {
        match state {
            MemoryRecordState::Current => Self::Current,
            MemoryRecordState::Closed => Self::Closed,
            MemoryRecordState::Archived => Self::Archived,
            MemoryRecordState::Background => Self::Background,
        }
    }

    /// Stable compact output code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Current => "MEM001",
            Self::Closed => "MEM002",
            Self::Archived => "MEM003",
            Self::Background => "MEM004",
        }
    }

    /// Agent-facing severity derived from official Org lifecycle evidence.
    pub const fn severity(&self) -> AgentMemorySeverity {
        match self {
            Self::Current => AgentMemorySeverity::Action,
            Self::Closed | Self::Archived => AgentMemorySeverity::Suppressed,
            Self::Background => AgentMemorySeverity::Info,
        }
    }

    const fn title(&self) -> &'static str {
        match self {
            Self::Current => "Current memory",
            Self::Closed => "Closed memory",
            Self::Archived => "Archived memory",
            Self::Background => "Background memory",
        }
    }

    const fn next_action(&self) -> &'static str {
        match self {
            Self::Current => {
                "Allow this fact into current decisions unless a caller profile narrows scope."
            }
            Self::Closed => {
                "Keep as historical evidence; do not promote as current without newer evidence."
            }
            Self::Archived => {
                "Keep as archived evidence; exclude from active decisions by default."
            }
            Self::Background => "Use as context only; do not let it drive action by itself.",
        }
    }
}

/// Agent-facing memory card severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentMemorySeverity {
    Action,
    Suppressed,
    Info,
}

impl AgentMemorySeverity {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Action => "Action",
            Self::Suppressed => "Suppressed",
            Self::Info => "Info",
        }
    }
}

pub(crate) fn is_done_todo(todo: &Option<TodoKeyword>) -> bool {
    matches!(
        todo,
        Some(TodoKeyword {
            state: TodoState::Done,
            ..
        })
    )
}
