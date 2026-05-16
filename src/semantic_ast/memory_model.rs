//! Agent-facing memory projection types derived from ordinary Org semantics.

use super::model::{ParsedAnnotation, SourcePosition, TodoKeyword, TodoState};
use super::timestamp_model::TimestampKind;

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
    pub authority: Vec<MemoryAuthorityReason>,
    pub title: String,
    pub todo: Option<TodoKeyword>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub anchor: Option<String>,
    pub evidence: Vec<MemoryEvidence>,
    pub links: Vec<MemoryLink>,
}

impl AgentMemoryCard {
    pub(crate) fn from_records(records: Vec<MemoryRecord>) -> Vec<Self> {
        let current_keys = records
            .iter()
            .filter(|record| record.state == MemoryRecordState::Current)
            .flat_map(memory_authority_keys)
            .collect::<Vec<_>>();

        records
            .into_iter()
            .map(|record| {
                let mut extra = Vec::new();
                if is_historical_state(record.state) {
                    push_authority(
                        &mut extra,
                        MemoryAuthorityKind::StaleCandidate,
                        "historical state makes this a stale candidate for current decisions",
                    );
                }
                if is_historical_state(record.state)
                    && memory_authority_keys(&record)
                        .into_iter()
                        .any(|key| current_keys.iter().any(|current| current == &key))
                {
                    push_authority(
                        &mut extra,
                        MemoryAuthorityKind::SupersededCandidate,
                        "a current memory card with the same anchor or title may supersede this fact",
                    );
                }
                Self::from_record_with_extra_authority(record, extra)
            })
            .collect()
    }

    fn from_record_with_extra_authority(
        record: MemoryRecord,
        mut extra_authority: Vec<MemoryAuthorityReason>,
    ) -> Self {
        let decision = AgentMemoryDecision::from_state(record.state);
        let mut authority = memory_authority_reasons(record.state, &record.evidence);
        authority.append(&mut extra_authority);
        Self {
            source: record.source,
            decision,
            authority,
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
        if !self.authority.is_empty() {
            output.push_str("authority: ");
            output.push_str(
                &self
                    .authority
                    .iter()
                    .map(|reason| reason.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
            );
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

/// Agent-facing reason that explains how memory evidence may influence a decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryAuthorityReason {
    pub kind: MemoryAuthorityKind,
    pub message: String,
}

/// Coarse authority category for agent memory consumption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryAuthorityKind {
    Current,
    Closed,
    Archived,
    Background,
    Identity,
    Temporal,
    Lifecycle,
    Attachment,
    Habit,
    Repeat,
    StaleCandidate,
    SupersededCandidate,
}

/// Kind of official Org evidence attached to a memory record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryEvidenceKind {
    TodoState,
    ArchiveTag,
    ArchiveLocation,
    ArchiveProperty,
    AttachmentTag,
    AttachmentDirectory,
    HabitStyle,
    HabitLastRepeat,
    HabitRepeater,
    Identity { key: String },
    Property { key: String },
    Scheduled,
    Deadline,
    Closed,
    Timestamp { kind: TimestampKind },
    Logbook,
    Drawer { name: String },
    Clock,
    Link,
    AttachmentLink,
    Lifecycle(MemoryLifecycleKind),
}

impl MemoryEvidenceKind {
    fn title(&self) -> String {
        match self {
            Self::TodoState => "TODO state".to_string(),
            Self::ArchiveTag => "ARCHIVE tag".to_string(),
            Self::ArchiveLocation => "archive location".to_string(),
            Self::ArchiveProperty => "ARCHIVE property".to_string(),
            Self::AttachmentTag => "ATTACH tag".to_string(),
            Self::AttachmentDirectory => "attachment directory".to_string(),
            Self::HabitStyle => "habit style".to_string(),
            Self::HabitLastRepeat => "habit last repeat".to_string(),
            Self::HabitRepeater => "habit repeater".to_string(),
            Self::Identity { key } => format!("identity {key}"),
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
            Self::AttachmentLink => "attachment link".to_string(),
            Self::Lifecycle(kind) => format!("lifecycle {}", kind.title()),
        }
    }
}

fn memory_authority_reasons(
    state: MemoryRecordState,
    evidence: &[MemoryEvidence],
) -> Vec<MemoryAuthorityReason> {
    let mut reasons = Vec::new();
    match state {
        MemoryRecordState::Current => push_authority(
            &mut reasons,
            MemoryAuthorityKind::Current,
            "current Org task or planning evidence may enter active decisions",
        ),
        MemoryRecordState::Closed => push_authority(
            &mut reasons,
            MemoryAuthorityKind::Closed,
            "DONE or CLOSED evidence keeps this fact historical",
        ),
        MemoryRecordState::Archived => push_authority(
            &mut reasons,
            MemoryAuthorityKind::Archived,
            "ARCHIVE evidence suppresses active promotion by default",
        ),
        MemoryRecordState::Background => push_authority(
            &mut reasons,
            MemoryAuthorityKind::Background,
            "no active task lifecycle; use as background context",
        ),
    }

    for item in evidence {
        match &item.kind {
            MemoryEvidenceKind::ArchiveTag
            | MemoryEvidenceKind::ArchiveLocation
            | MemoryEvidenceKind::ArchiveProperty => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Archived,
                "archive metadata marks this as retained historical evidence",
            ),
            MemoryEvidenceKind::Closed => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Closed,
                "CLOSED timestamp blocks promotion as a fresh active fact",
            ),
            MemoryEvidenceKind::Identity { .. } => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Identity,
                "stable identity evidence lets agents correlate corrections across time",
            ),
            MemoryEvidenceKind::Scheduled
            | MemoryEvidenceKind::Deadline
            | MemoryEvidenceKind::Timestamp { .. } => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Temporal,
                "timestamp evidence gives this fact a bounded time context",
            ),
            MemoryEvidenceKind::Lifecycle(_) | MemoryEvidenceKind::Logbook => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Lifecycle,
                "LOGBOOK or lifecycle records explain how this fact changed over time",
            ),
            MemoryEvidenceKind::AttachmentTag
            | MemoryEvidenceKind::AttachmentDirectory
            | MemoryEvidenceKind::AttachmentLink => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Attachment,
                "attachment evidence provides supporting artifact context",
            ),
            MemoryEvidenceKind::HabitStyle => push_authority(
                &mut reasons,
                MemoryAuthorityKind::Habit,
                "habit evidence marks recurring cadence instead of a one-off fact",
            ),
            MemoryEvidenceKind::HabitLastRepeat | MemoryEvidenceKind::HabitRepeater => {
                push_authority(
                    &mut reasons,
                    MemoryAuthorityKind::Habit,
                    "habit evidence marks recurring cadence instead of a one-off fact",
                );
                push_authority(
                    &mut reasons,
                    MemoryAuthorityKind::Repeat,
                    "repeat evidence should be interpreted as cadence, not a timeless preference",
                );
            }
            MemoryEvidenceKind::TodoState
            | MemoryEvidenceKind::Property { .. }
            | MemoryEvidenceKind::Drawer { .. }
            | MemoryEvidenceKind::Clock
            | MemoryEvidenceKind::Link => {}
        }
    }
    reasons
}

fn push_authority(
    reasons: &mut Vec<MemoryAuthorityReason>,
    kind: MemoryAuthorityKind,
    message: &'static str,
) {
    if reasons.iter().any(|reason| reason.kind == kind) {
        return;
    }
    reasons.push(MemoryAuthorityReason {
        kind,
        message: message.to_string(),
    });
}

fn is_historical_state(state: MemoryRecordState) -> bool {
    matches!(
        state,
        MemoryRecordState::Closed | MemoryRecordState::Archived
    )
}

fn memory_authority_keys(record: &MemoryRecord) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(anchor) = &record.anchor {
        keys.push(format!("anchor:{}", anchor.to_ascii_lowercase()));
    }
    for property in &record.properties {
        if property.key.eq_ignore_ascii_case("ID") || property.key.eq_ignore_ascii_case("CUSTOM_ID")
        {
            keys.push(format!(
                "{}:{}",
                property.key.to_ascii_lowercase(),
                property.value.to_ascii_lowercase()
            ));
        }
    }
    keys.push(format!(
        "title:{}",
        record
            .title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase()
    ));
    keys
}

/// Lifecycle event category used by memory evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryLifecycleKind {
    StateChange,
    Note,
    Refile,
    Reschedule,
    Redeadline,
    Clock,
}

impl MemoryLifecycleKind {
    pub const fn title(self) -> &'static str {
        match self {
            Self::StateChange => "state change",
            Self::Note => "note",
            Self::Refile => "refile",
            Self::Reschedule => "reschedule",
            Self::Redeadline => "redeadline",
            Self::Clock => "clock",
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
