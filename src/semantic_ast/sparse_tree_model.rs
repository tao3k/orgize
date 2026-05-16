//! Sparse-tree style projections for document-local agent/search consumers.

use super::{
    AgendaMatchParseError, AgendaMatchQuery, Planning, Priority, SectionIndexArchive,
    SectionIndexAttachment, SectionIndexCategory, SectionIndexLifecycleRecord, SectionIndexLink,
    SectionIndexProperty, SectionIndexSource, SectionIndexSpecialProperty, SectionIndexTarget,
    TodoKeyword,
};

/// Query for document-local sparse-tree projections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseTreeQuery {
    pub(crate) include_comments: bool,
    pub(crate) include_done: bool,
    pub(crate) include_archived: bool,
    pub(crate) source_file: Option<String>,
    pub(crate) match_query: Option<AgendaMatchQuery>,
    pub(crate) text: Option<String>,
    pub(crate) explain_skips: bool,
}

impl SparseTreeQuery {
    /// Creates a query that keeps ordinary historical Org evidence visible.
    pub fn new() -> Self {
        Self {
            include_comments: false,
            include_done: true,
            include_archived: true,
            source_file: None,
            match_query: None,
            text: None,
            explain_skips: false,
        }
    }

    /// Includes or excludes COMMENT headlines.
    pub fn include_comments(mut self, include_comments: bool) -> Self {
        self.include_comments = include_comments;
        self
    }

    /// Includes or excludes DONE-state headlines.
    pub fn include_done(mut self, include_done: bool) -> Self {
        self.include_done = include_done;
        self
    }

    /// Includes or excludes archived headlines.
    pub fn include_archived(mut self, include_archived: bool) -> Self {
        self.include_archived = include_archived;
        self
    }

    /// Adds caller source-file context for official `FILE` special properties.
    pub fn source_file(mut self, source_file: impl Into<String>) -> Self {
        self.source_file = Some(source_file.into());
        self
    }

    /// Adds an Org Agenda-style match expression.
    pub fn match_expression(
        mut self,
        expression: impl AsRef<str>,
    ) -> Result<Self, AgendaMatchParseError> {
        self.match_query = Some(AgendaMatchQuery::parse(expression)?);
        Ok(self)
    }

    /// Adds a document-local text predicate over titles, body slices, metadata,
    /// links, and targets.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            self.text = Some(trimmed.to_string());
        }
        self
    }

    /// Records skipped section receipts for audit/debug consumers.
    ///
    /// The default keeps skipped rows out of the projection to avoid noisy
    /// agent contexts. Enable this when a downstream caller needs to explain
    /// why a document-local predicate did not surface a section.
    pub fn explain_skips(mut self, explain_skips: bool) -> Self {
        self.explain_skips = explain_skips;
        self
    }

    pub(crate) fn has_predicate(&self) -> bool {
        self.match_query.is_some() || self.text.is_some()
    }
}

impl Default for SparseTreeQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Sparse-tree projection over one parsed Org document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseTreeProjection {
    pub total_candidates: usize,
    pub cards: Vec<SparseTreeCard>,
    pub skipped: Vec<SparseTreeSkip>,
}

impl SparseTreeProjection {
    /// Returns true when no sparse-tree card is visible for the query.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Renders sparse-tree cards as compact text for coding agents.
    pub fn to_compact_text(&self, path: &str) -> String {
        if self.cards.is_empty() && self.skipped.is_empty() {
            return "[ok] orgize sparse tree\n".to_string();
        }

        let mut parts = Vec::new();
        parts.extend(self.cards.iter().map(|card| card.to_compact_text(path)));
        parts.extend(self.skipped.iter().map(|skip| skip.to_compact_text(path)));
        parts.join("\n")
    }
}

/// One sparse-tree card backed by a single Org section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseTreeCard {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub matches: Vec<SparseTreeMatch>,
    pub receipts: Vec<SparseTreeReceipt>,
    pub preview: Option<String>,
    pub todo: Option<TodoKeyword>,
    pub priority: Priority,
    pub category: Option<SectionIndexCategory>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub properties: Vec<SectionIndexProperty>,
    pub special_properties: Vec<SectionIndexSpecialProperty>,
    pub planning: Planning,
    pub archive: SectionIndexArchive,
    pub attachment: SectionIndexAttachment,
    pub links: Vec<SectionIndexLink>,
    pub targets: Vec<SectionIndexTarget>,
    pub lifecycle: Vec<SectionIndexLifecycleRecord>,
}

impl SparseTreeCard {
    fn to_compact_text(&self, path: &str) -> String {
        let mut output = String::new();
        output.push_str("[SPARSE001] Match: ");
        output.push_str(&self.title);
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        output.push_str("outline: ");
        output.push_str(&self.outline_path.join(" / "));
        output.push('\n');
        if let Some(todo) = &self.todo {
            output.push_str("state: ");
            output.push_str(&todo.name);
            output.push('\n');
        }
        output.push_str("priority: ");
        output.push_str(&self.priority.effective_text());
        output.push('\n');
        if !self.effective_tags.is_empty() {
            output.push_str("tags: ");
            output.push_str(&self.effective_tags.join(":"));
            output.push('\n');
        }
        let planning = compact_planning(&self.planning);
        if !planning.is_empty() {
            output.push_str("planning: ");
            output.push_str(&planning.join(", "));
            output.push('\n');
        }
        output.push_str("matches: ");
        output.push_str(&compact_matches(&self.matches));
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
        if let Some(preview) = &self.preview {
            output.push_str("preview: ");
            output.push_str(preview);
            output.push('\n');
        }
        if !self.links.is_empty() {
            output.push_str("links: ");
            output.push_str(
                &self
                    .links
                    .iter()
                    .map(|link| link.path.as_str())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if self.links.len() > 3 {
                output.push_str(", ...");
            }
            output.push('\n');
        }
        output.push_str("contract: Derived from official Org sparse-tree/search constructs; no custom source syntax is required.");
        output.push('\n');
        output
    }
}

/// One sparse-tree section skipped from the card list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseTreeSkip {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub reason: SparseTreeSkipReason,
    pub receipts: Vec<SparseTreeReceipt>,
}

impl SparseTreeSkip {
    fn to_compact_text(&self, path: &str) -> String {
        let mut output = String::new();
        output.push_str("[SPARSE_SKIP] ");
        output.push_str(&self.title);
        output.push('\n');
        output.push_str("@ ");
        output.push_str(path);
        output.push(':');
        output.push_str(&self.source.start.line.to_string());
        output.push(':');
        output.push_str(&self.source.start.column.to_string());
        output.push('\n');
        output.push_str("outline: ");
        output.push_str(&self.outline_path.join(" / "));
        output.push('\n');
        output.push_str("reason: ");
        output.push_str(self.reason.as_str());
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
        output
    }
}

/// Why a sparse-tree candidate was skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseTreeSkipReason {
    Comment,
    Archived,
    Done,
    MatchExpression,
    Text,
}

impl SparseTreeSkipReason {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Archived => "archived",
            Self::Done => "done",
            Self::MatchExpression => "matchExpression",
            Self::Text => "text",
        }
    }
}

/// One source-grounded reason a sparse-tree card matched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseTreeMatch {
    pub source: SectionIndexSource,
    pub kind: SparseTreeMatchKind,
    pub key: Option<String>,
    pub value: String,
}

/// One audit receipt for a sparse-tree decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseTreeReceipt {
    pub kind: SparseTreeReceiptKind,
    pub message: String,
}

/// Stable receipt kind for sparse-tree decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseTreeReceiptKind {
    Candidate,
    VisibilityFilterPassed,
    MatchExpressionMatched,
    TextMatched,
    DefaultAllMatched,
    Accepted,
    SkippedComment,
    SkippedArchived,
    SkippedDone,
    SkippedMatchExpression,
    SkippedText,
}

impl SparseTreeReceiptKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::VisibilityFilterPassed => "visibilityFilterPassed",
            Self::MatchExpressionMatched => "matchExpressionMatched",
            Self::TextMatched => "textMatched",
            Self::DefaultAllMatched => "defaultAllMatched",
            Self::Accepted => "accepted",
            Self::SkippedComment => "skippedComment",
            Self::SkippedArchived => "skippedArchived",
            Self::SkippedDone => "skippedDone",
            Self::SkippedMatchExpression => "skippedMatchExpression",
            Self::SkippedText => "skippedText",
        }
    }
}

/// Stable match-reason categories for sparse-tree projections.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SparseTreeMatchKind {
    Query,
    Title,
    Body,
    Tag,
    Property,
    SpecialProperty,
    Planning,
    Priority,
    Link,
    Target,
}

impl SparseTreeMatchKind {
    /// Stable compact label for agent and DTO consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Title => "title",
            Self::Body => "body",
            Self::Tag => "tag",
            Self::Property => "property",
            Self::SpecialProperty => "specialProperty",
            Self::Planning => "planning",
            Self::Priority => "priority",
            Self::Link => "link",
            Self::Target => "target",
        }
    }
}

fn compact_planning(planning: &Planning) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(timestamp) = &planning.scheduled {
        parts.push(format!("scheduled={}", timestamp.raw));
    }
    if let Some(timestamp) = &planning.deadline {
        parts.push(format!("deadline={}", timestamp.raw));
    }
    if let Some(timestamp) = &planning.closed {
        parts.push(format!("closed={}", timestamp.raw));
    }
    parts
}

fn compact_matches(matches: &[SparseTreeMatch]) -> String {
    if matches.is_empty() {
        return "query".to_string();
    }
    let mut parts = matches
        .iter()
        .take(4)
        .map(|matched| {
            let value = truncate_text(&matched.value, 48);
            match &matched.key {
                Some(key) => format!("{}:{}={}", matched.kind.as_str(), key, value),
                None => format!("{}={}", matched.kind.as_str(), value),
            }
        })
        .collect::<Vec<_>>();
    if matches.len() > 4 {
        parts.push(format!("+{} more", matches.len() - 4));
    }
    parts.join(", ")
}

pub(crate) fn truncate_text(value: &str, max_chars: usize) -> String {
    let mut truncated = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index >= max_chars {
            truncated.push_str("...");
            return truncated;
        }
        truncated.push(ch);
    }
    truncated
}
