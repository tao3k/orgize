//! Agent-facing capture plan DTOs.

use super::{AgendaDate, AgendaTime};

/// Structured request for rendering a non-mutating native Org capture plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureRequest {
    pub kind: AgentCaptureKind,
    pub title: String,
    pub body: Option<String>,
    pub target: AgentCaptureTarget,
    pub source: AgentCaptureSource,
    pub captured_at: Option<AgentCaptureTimestamp>,
    pub tags: Vec<String>,
    pub properties: Vec<AgentCaptureProperty>,
    pub quote: Option<String>,
    pub links: Vec<AgentCaptureLink>,
    pub memory_policy: AgentCaptureMemoryPolicy,
    pub requires_confirmation: bool,
}

impl AgentCaptureRequest {
    /// Creates a request with conservative conversation/inbox defaults.
    pub fn new(kind: AgentCaptureKind, title: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            body: None,
            target: AgentCaptureTarget::inbox(),
            source: AgentCaptureSource::conversation(),
            captured_at: None,
            tags: Vec::new(),
            properties: Vec::new(),
            quote: None,
            links: Vec::new(),
            memory_policy: AgentCaptureMemoryPolicy::None,
            requires_confirmation: true,
        }
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn target(mut self, target: AgentCaptureTarget) -> Self {
        self.target = target;
        self
    }

    pub fn source(mut self, source: AgentCaptureSource) -> Self {
        self.source = source;
        self
    }

    pub fn captured_at(mut self, captured_at: AgentCaptureTimestamp) -> Self {
        self.captured_at = Some(captured_at);
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push(AgentCaptureProperty {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn quote(mut self, quote: impl Into<String>) -> Self {
        self.quote = Some(quote.into());
        self
    }

    pub fn link(mut self, url: impl Into<String>, label: impl Into<String>) -> Self {
        self.links.push(AgentCaptureLink {
            url: url.into(),
            label: Some(label.into()),
        });
        self
    }

    pub fn memory_policy(mut self, memory_policy: AgentCaptureMemoryPolicy) -> Self {
        self.memory_policy = memory_policy;
        self
    }

    pub fn requires_confirmation(mut self, requires_confirmation: bool) -> Self {
        self.requires_confirmation = requires_confirmation;
        self
    }

    /// Renders this request into a non-mutating native Org capture plan.
    pub fn plan(&self) -> AgentCapturePlan {
        super::capture::agent_capture_plan(self)
    }
}

/// Agent-level capture intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureKind {
    Idea,
    ArticleNote,
    Task,
    Decision,
    Preference,
    Correction,
    MemoryCandidate,
    Evidence,
    Note,
}

impl AgentCaptureKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idea => "idea",
            Self::ArticleNote => "articleNote",
            Self::Task => "task",
            Self::Decision => "decision",
            Self::Preference => "preference",
            Self::Correction => "correction",
            Self::MemoryCandidate => "memoryCandidate",
            Self::Evidence => "evidence",
            Self::Note => "note",
        }
    }

    pub(crate) const fn todo_keyword(self) -> Option<&'static str> {
        match self {
            Self::Task => Some("TODO"),
            _ => None,
        }
    }
}

/// Target intent for a downstream runtime to apply after review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureTarget {
    pub kind: AgentCaptureTargetKind,
    pub source_file: Option<String>,
    pub outline_path: Vec<String>,
    pub date: Option<AgendaDate>,
    pub insert_position: AgentCaptureInsertPosition,
}

impl AgentCaptureTarget {
    pub fn inbox() -> Self {
        Self {
            kind: AgentCaptureTargetKind::Inbox,
            source_file: None,
            outline_path: vec!["Inbox".to_string()],
            date: None,
            insert_position: AgentCaptureInsertPosition::Append,
        }
    }

    pub fn datetree(date: AgendaDate) -> Self {
        Self {
            kind: AgentCaptureTargetKind::Datetree,
            source_file: None,
            outline_path: Vec::new(),
            date: Some(date),
            insert_position: AgentCaptureInsertPosition::Append,
        }
    }

    pub fn outline_path(path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            kind: AgentCaptureTargetKind::OutlinePath,
            source_file: None,
            outline_path: path.into_iter().map(Into::into).collect(),
            date: None,
            insert_position: AgentCaptureInsertPosition::Append,
        }
    }

    pub fn current_section() -> Self {
        Self {
            kind: AgentCaptureTargetKind::CurrentSection,
            source_file: None,
            outline_path: Vec::new(),
            date: None,
            insert_position: AgentCaptureInsertPosition::Append,
        }
    }

    pub fn source_file(mut self, source_file: impl Into<String>) -> Self {
        self.source_file = Some(source_file.into());
        self
    }

    pub fn insert_position(mut self, insert_position: AgentCaptureInsertPosition) -> Self {
        self.insert_position = insert_position;
        self
    }
}

/// Stable target categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureTargetKind {
    Inbox,
    Datetree,
    OutlinePath,
    CurrentSection,
}

impl AgentCaptureTargetKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Datetree => "datetree",
            Self::OutlinePath => "outlinePath",
            Self::CurrentSection => "currentSection",
        }
    }
}

/// Where a runtime should place the entry relative to the target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureInsertPosition {
    Append,
    Prepend,
    FirstChild,
    LastChild,
}

impl AgentCaptureInsertPosition {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::Prepend => "prepend",
            Self::FirstChild => "firstChild",
            Self::LastChild => "lastChild",
        }
    }
}

/// Provenance for a capture request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureSource {
    pub kind: AgentCaptureSourceKind,
    pub actor: Option<String>,
    pub uri: Option<String>,
    pub label: Option<String>,
}

impl AgentCaptureSource {
    pub fn conversation() -> Self {
        Self {
            kind: AgentCaptureSourceKind::Conversation,
            actor: Some("user".to_string()),
            uri: None,
            label: None,
        }
    }

    pub fn url(uri: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            kind: AgentCaptureSourceKind::Url,
            actor: None,
            uri: Some(uri.into()),
            label: Some(label.into()),
        }
    }

    pub fn actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }
}

/// Stable source categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureSourceKind {
    Conversation,
    Url,
    File,
    Selection,
    Article,
    Code,
    Other,
}

impl AgentCaptureSourceKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Url => "url",
            Self::File => "file",
            Self::Selection => "selection",
            Self::Article => "article",
            Self::Code => "code",
            Self::Other => "other",
        }
    }
}

/// Inactive timestamp evidence supplied by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgentCaptureTimestamp {
    pub date: AgendaDate,
    pub time: Option<AgendaTime>,
}

impl AgentCaptureTimestamp {
    pub const fn new(date: AgendaDate) -> Self {
        Self { date, time: None }
    }

    pub const fn with_time(date: AgendaDate, time: AgendaTime) -> Self {
        Self {
            date,
            time: Some(time),
        }
    }
}

/// Ordinary Org property to add to the generated property drawer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureProperty {
    pub key: String,
    pub value: String,
}

/// Ordinary Org link to render in the entry body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureLink {
    pub url: String,
    pub label: Option<String>,
}

/// How downstream memory policy should treat this captured evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureMemoryPolicy {
    None,
    Candidate,
    Background,
    Decision,
    Transient,
    Supersedes,
}

impl AgentCaptureMemoryPolicy {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Candidate => "candidate",
            Self::Background => "background",
            Self::Decision => "decision",
            Self::Transient => "transient",
            Self::Supersedes => "supersedes",
        }
    }
}

/// Non-mutating capture plan rendered from an Agent request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCapturePlan {
    pub target: AgentCaptureTarget,
    pub org_entry: String,
    pub receipts: Vec<AgentCaptureReceipt>,
    pub warnings: Vec<AgentCaptureWarning>,
    pub requires_confirmation: bool,
}

impl AgentCapturePlan {
    /// Renders a non-mutating native Org capture plan from a request.
    pub fn from_request(request: &AgentCaptureRequest) -> Self {
        request.plan()
    }
}

/// Why a capture plan is safe or how it should be interpreted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureReceipt {
    pub kind: AgentCaptureReceiptKind,
    pub message: String,
}

/// Stable receipt categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureReceiptKind {
    NonMutating,
    NativeOrgEntry,
    AgentInterpreted,
    SourceProvenance,
    MemoryPolicy,
    RequiresConfirmation,
}

impl AgentCaptureReceiptKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NonMutating => "nonMutating",
            Self::NativeOrgEntry => "nativeOrgEntry",
            Self::AgentInterpreted => "agentInterpreted",
            Self::SourceProvenance => "sourceProvenance",
            Self::MemoryPolicy => "memoryPolicy",
            Self::RequiresConfirmation => "requiresConfirmation",
        }
    }
}

/// Non-fatal capture plan warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentCaptureWarning {
    pub kind: AgentCaptureWarningKind,
    pub message: String,
}

/// Stable warning categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCaptureWarningKind {
    EmptyTitle,
    EmptyBody,
    SanitizedTag,
    SanitizedPropertyKey,
    RuntimeOwnedTarget,
}

impl AgentCaptureWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyTitle => "emptyTitle",
            Self::EmptyBody => "emptyBody",
            Self::SanitizedTag => "sanitizedTag",
            Self::SanitizedPropertyKey => "sanitizedPropertyKey",
            Self::RuntimeOwnedTarget => "runtimeOwnedTarget",
        }
    }
}
