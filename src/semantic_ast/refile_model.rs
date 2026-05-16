//! Non-executing refile target and movement-plan projections.

use super::SectionIndexSource;

/// Query for document-local refile target discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileTargetQuery {
    pub(crate) source_file: Option<String>,
    pub(crate) outline_path_mode: RefileOutlinePathMode,
    pub(crate) specs: Vec<RefileTargetSpec>,
}

impl RefileTargetQuery {
    /// Creates a query that mirrors Org's current-buffer default: level 1
    /// headlines are refile targets when no explicit target spec is provided.
    pub fn new() -> Self {
        Self {
            source_file: None,
            outline_path_mode: RefileOutlinePathMode::None,
            specs: Vec::new(),
        }
    }

    /// Adds caller source-file context for target display and `FILE` metadata.
    pub fn source_file(mut self, source_file: impl Into<String>) -> Self {
        self.source_file = Some(source_file.into());
        self
    }

    /// Selects how target names should be displayed to agent/UI consumers.
    pub fn outline_path_mode(mut self, mode: RefileOutlinePathMode) -> Self {
        self.outline_path_mode = mode;
        self
    }

    /// Adds an official-style target spec.
    pub fn spec(mut self, spec: RefileTargetSpec) -> Self {
        self.specs.push(spec);
        self
    }

    /// Considers all headlines as targets.
    pub fn all(self) -> Self {
        self.spec(RefileTargetSpec::All)
    }

    /// Considers headlines with a local tag as targets.
    pub fn tag(self, tag: impl Into<String>) -> Self {
        self.spec(RefileTargetSpec::Tag(tag.into()))
    }

    /// Considers headlines with a TODO keyword as targets.
    pub fn todo(self, keyword: impl Into<String>) -> Self {
        self.spec(RefileTargetSpec::Todo(keyword.into()))
    }

    /// Considers headlines at exactly `level` as targets.
    pub fn level(self, level: usize) -> Self {
        self.spec(RefileTargetSpec::Level(level))
    }

    /// Considers headlines up to and including `level` as targets.
    pub fn max_level(self, level: usize) -> Self {
        self.spec(RefileTargetSpec::MaxLevel(level))
    }

    /// Preserves an official regexp target spec. The initial projection records
    /// this as unsupported instead of approximating Emacs regexp semantics.
    pub fn regexp(self, pattern: impl Into<String>) -> Self {
        self.spec(RefileTargetSpec::Regexp(pattern.into()))
    }

    pub(crate) fn effective_specs(&self) -> Vec<RefileTargetSpec> {
        if self.specs.is_empty() {
            vec![RefileTargetSpec::Level(1)]
        } else {
            self.specs.clone()
        }
    }
}

impl Default for RefileTargetQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Official-style refile target selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefileTargetSpec {
    All,
    Tag(String),
    Todo(String),
    Level(usize),
    MaxLevel(usize),
    Regexp(String),
}

impl RefileTargetSpec {
    /// Stable selector kind for DTOs and compact receipts.
    pub fn kind(&self) -> RefileTargetSpecKind {
        match self {
            Self::All => RefileTargetSpecKind::All,
            Self::Tag(_) => RefileTargetSpecKind::Tag,
            Self::Todo(_) => RefileTargetSpecKind::Todo,
            Self::Level(_) => RefileTargetSpecKind::Level,
            Self::MaxLevel(_) => RefileTargetSpecKind::MaxLevel,
            Self::Regexp(_) => RefileTargetSpecKind::Regexp,
        }
    }

    /// Optional selector payload.
    pub fn value(&self) -> Option<String> {
        match self {
            Self::All => None,
            Self::Tag(value) | Self::Todo(value) | Self::Regexp(value) => Some(value.clone()),
            Self::Level(level) | Self::MaxLevel(level) => Some(level.to_string()),
        }
    }
}

/// Stable refile target selector kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefileTargetSpecKind {
    All,
    Tag,
    Todo,
    Level,
    MaxLevel,
    Regexp,
}

impl RefileTargetSpecKind {
    /// Stable string label for DTOs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Tag => "tag",
            Self::Todo => "todo",
            Self::Level => "level",
            Self::MaxLevel => "maxLevel",
            Self::Regexp => "regexp",
        }
    }
}

/// How target names should include outline/file context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefileOutlinePathMode {
    None,
    Outline,
    File,
    FullFilePath,
    BufferName,
    Title,
}

impl RefileOutlinePathMode {
    /// Stable string label for DTOs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Outline => "outline",
            Self::File => "file",
            Self::FullFilePath => "fullFilePath",
            Self::BufferName => "bufferName",
            Self::Title => "title",
        }
    }
}

/// Source-grounded refile target index for one parsed Org document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileTargetIndex {
    pub source_file: Option<String>,
    pub outline_path_mode: RefileOutlinePathMode,
    pub specs: Vec<RefileTargetSpec>,
    pub targets: Vec<RefileTarget>,
    pub warnings: Vec<RefileWarning>,
}

/// One refile target candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileTarget {
    pub source_file: Option<String>,
    pub source: SectionIndexSource,
    pub level: usize,
    pub title: String,
    pub outline_path: Vec<String>,
    pub display: String,
    pub receipts: Vec<RefileTargetReceipt>,
}

/// Why one target candidate was accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileTargetReceipt {
    pub spec: RefileTargetSpec,
    pub message: String,
}

/// Request for a non-mutating refile movement plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefilePlanRequest {
    pub source_file: Option<String>,
    pub source_outline_path: Vec<String>,
    pub target_outline_path: Vec<String>,
    pub action: RefileAction,
    pub insert_position: RefileInsertPosition,
    pub parent_creation: RefileParentCreationMode,
}

impl RefilePlanRequest {
    /// Creates a move plan from one outline path to another.
    pub fn new(
        source_outline_path: impl IntoIterator<Item = impl Into<String>>,
        target_outline_path: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            source_file: None,
            source_outline_path: source_outline_path.into_iter().map(Into::into).collect(),
            target_outline_path: target_outline_path.into_iter().map(Into::into).collect(),
            action: RefileAction::Move,
            insert_position: RefileInsertPosition::LastChild,
            parent_creation: RefileParentCreationMode::Never,
        }
    }

    /// Adds caller source-file context.
    pub fn source_file(mut self, source_file: impl Into<String>) -> Self {
        self.source_file = Some(source_file.into());
        self
    }

    /// Selects the refile action label.
    pub fn action(mut self, action: RefileAction) -> Self {
        self.action = action;
        self
    }

    /// Selects the non-mutating insertion intent under the target.
    pub fn insert_position(mut self, insert_position: RefileInsertPosition) -> Self {
        self.insert_position = insert_position;
        self
    }

    /// Selects how missing final target headings should be planned.
    pub fn parent_creation(mut self, mode: RefileParentCreationMode) -> Self {
        self.parent_creation = mode;
        self
    }

    /// Plans creation of one missing final target heading under an existing parent.
    pub fn allow_creating_parent_nodes(self) -> Self {
        self.parent_creation(RefileParentCreationMode::Plan)
    }

    /// Plans creation of one missing final target heading and marks it as confirmation-gated.
    pub fn confirm_creating_parent_nodes(self) -> Self {
        self.parent_creation(RefileParentCreationMode::Confirm)
    }
}

/// Refile action requested by a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefileAction {
    Move,
    Copy,
    Goto,
}

impl RefileAction {
    /// Stable string label for DTOs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Copy => "copy",
            Self::Goto => "goto",
        }
    }
}

/// Non-mutating insertion intent under the target heading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefileInsertPosition {
    LastChild,
    FirstChild,
}

impl RefileInsertPosition {
    /// Stable string label for DTOs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LastChild => "lastChild",
            Self::FirstChild => "firstChild",
        }
    }
}

/// Whether a refile plan may describe creation of a missing final target node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefileParentCreationMode {
    Never,
    Plan,
    Confirm,
}

impl RefileParentCreationMode {
    /// Stable string label for DTOs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Plan => "plan",
            Self::Confirm => "confirm",
        }
    }
}

/// Non-executing refile plan resolved against one parsed document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefilePlan {
    pub source_file: Option<String>,
    pub action: RefileAction,
    pub insert_position: RefileInsertPosition,
    pub parent_creation: RefileParentCreationMode,
    pub source: Option<RefilePlanSection>,
    pub target: Option<RefileTarget>,
    pub created_target: Option<RefileCreateParentPlan>,
    pub receipts: Vec<RefilePlanReceipt>,
    pub warnings: Vec<RefileWarning>,
}

/// Non-mutating plan for a missing final target heading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileCreateParentPlan {
    pub source_file: Option<String>,
    pub existing_parent: RefileTarget,
    pub target_outline_path: Vec<String>,
    pub nodes: Vec<RefileCreateParentNode>,
    pub requires_confirmation: bool,
}

/// One missing heading that Org would insert under an existing parent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileCreateParentNode {
    pub title: String,
    pub level: usize,
    pub outline_path: Vec<String>,
    pub display: String,
}

/// Section participating in a refile plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefilePlanSection {
    pub source_file: Option<String>,
    pub source: SectionIndexSource,
    pub level: usize,
    pub title: String,
    pub outline_path: Vec<String>,
    pub local_ids: Vec<String>,
}

/// A plan receipt visible to agents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefilePlanReceipt {
    pub kind: RefilePlanReceiptKind,
    pub message: String,
}

/// Stable plan receipt category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefilePlanReceiptKind {
    SourceResolved,
    TargetResolved,
    InsertPositionResolved,
    ParentCreationPlanned,
    ParentCreationRequiresConfirmation,
    NonMutating,
}

impl RefilePlanReceiptKind {
    /// Stable string label for DTOs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceResolved => "sourceResolved",
            Self::TargetResolved => "targetResolved",
            Self::InsertPositionResolved => "insertPositionResolved",
            Self::ParentCreationPlanned => "parentCreationPlanned",
            Self::ParentCreationRequiresConfirmation => "parentCreationRequiresConfirmation",
            Self::NonMutating => "nonMutating",
        }
    }
}

/// Warning emitted by refile target indexing or planning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefileWarning {
    pub kind: RefileWarningKind,
    pub message: String,
}

/// Stable refile warning category.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefileWarningKind {
    UnsupportedRegexp,
    DuplicateDisplay,
    SourceNotFound,
    TargetNotFound,
    AmbiguousSource,
    AmbiguousTarget,
    ParentNotFound,
    AmbiguousParent,
    SameSourceAndTarget,
    TargetInsideSource,
    CopyMayDuplicateId,
}

impl RefileWarningKind {
    /// Stable string label for DTOs.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedRegexp => "unsupportedRegexp",
            Self::DuplicateDisplay => "duplicateDisplay",
            Self::SourceNotFound => "sourceNotFound",
            Self::TargetNotFound => "targetNotFound",
            Self::AmbiguousSource => "ambiguousSource",
            Self::AmbiguousTarget => "ambiguousTarget",
            Self::ParentNotFound => "parentNotFound",
            Self::AmbiguousParent => "ambiguousParent",
            Self::SameSourceAndTarget => "sameSourceAndTarget",
            Self::TargetInsideSource => "targetInsideSource",
            Self::CopyMayDuplicateId => "copyMayDuplicateId",
        }
    }
}
