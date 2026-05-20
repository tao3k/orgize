//! Filesystem-aware attachment inventory DTOs.

use super::{AttachmentDirectorySource, AttachmentLink, SectionIndexSource};

/// Opt-in attachment inventory options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInventoryOptions {
    pub base_dir: String,
    pub attach_id_dir: String,
    pub check_vcs: bool,
    pub check_annex: bool,
    pub archive_delete_policy: AttachmentArchiveDeletePolicy,
    pub scan_orphans: bool,
}

impl AttachmentInventoryOptions {
    /// Creates inventory options rooted at a filesystem directory.
    pub fn new(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
            attach_id_dir: "data".to_string(),
            check_vcs: false,
            check_annex: false,
            archive_delete_policy: AttachmentArchiveDeletePolicy::NotConfigured,
            scan_orphans: false,
        }
    }

    /// Sets the effective `org-attach-id-dir` root for ID-derived attachment
    /// directories. The default keeps the existing `data/` layout.
    pub fn attach_id_dir(mut self, attach_id_dir: impl Into<String>) -> Self {
        self.attach_id_dir = attach_id_dir.into();
        self
    }

    /// Enables or disables git status checks for discovered paths.
    pub fn check_vcs(mut self, check_vcs: bool) -> Self {
        self.check_vcs = check_vcs;
        self
    }

    /// Enables or disables git-annex availability checks when VCS checks run.
    pub fn check_annex(mut self, check_annex: bool) -> Self {
        self.check_annex = check_annex;
        self
    }

    /// Sets the caller-known `org-attach-archive-delete` policy.
    pub fn archive_delete_policy(mut self, policy: AttachmentArchiveDeletePolicy) -> Self {
        self.archive_delete_policy = policy;
        self
    }

    /// Enables or disables direct directory scans for orphan/stale attachment
    /// sync advice.
    pub fn scan_orphans(mut self, scan_orphans: bool) -> Self {
        self.scan_orphans = scan_orphans;
        self
    }
}

/// Filesystem-aware attachment inventory for one parsed document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttachmentInventory {
    pub entries: Vec<AttachmentInventoryEntry>,
    pub display: Vec<AttachmentDisplayRecord>,
    pub sync_plan: AttachmentSyncPlan,
    pub archive_advice: Vec<AttachmentArchiveAdvice>,
    pub warnings: Vec<AttachmentInventoryWarning>,
}

/// One attachment directory or link inventory entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInventoryEntry {
    pub source: SectionIndexSource,
    pub section_title: String,
    pub kind: AttachmentInventoryEntryKind,
    pub path: String,
    pub absolute_path: String,
    pub exists: bool,
    pub vcs: AttachmentVcsEvidence,
}

/// Non-mutating advice for archived sections whose attachments may be deleted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentArchiveAdvice {
    pub source: SectionIndexSource,
    pub section_title: String,
    pub policy: AttachmentArchiveDeletePolicy,
    pub path: String,
    pub message: String,
}

/// Attachment link record ready for a UI/gallery layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentDisplayRecord {
    pub source: SectionIndexSource,
    pub section_title: String,
    pub outline_path: Vec<String>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub attachment_id: Option<AttachmentDisplayId>,
    pub directory_path: AttachmentDisplayDirectoryPath,
    pub link_path: AttachmentDisplayLinkPath,
    pub absolute_path: AttachmentDisplayAbsolutePath,
    pub exists: bool,
    pub media_kind: AttachmentDisplayMediaKind,
}

/// Org attachment ID associated with a display record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentDisplayId(String);

impl AttachmentDisplayId {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the source-backed ID text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Effective attachment directory used for a display record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentDisplayDirectoryPath(String);

impl AttachmentDisplayDirectoryPath {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the directory path text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Attachment link file path used for a display record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentDisplayLinkPath(String);

impl AttachmentDisplayLinkPath {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the link path text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Resolved absolute path for a display record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentDisplayAbsolutePath(String);

impl AttachmentDisplayAbsolutePath {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the absolute path text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Stable media kind inferred from an attachment file name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentDisplayMediaKind {
    Image,
    Video,
    Audio,
    Pdf,
    Other,
}

impl AttachmentDisplayMediaKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Pdf => "pdf",
            Self::Other => "other",
        }
    }
}

/// Non-mutating attachment synchronization plan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttachmentSyncPlan {
    pub actions: Vec<AttachmentSyncAction>,
}

/// One proposed attachment synchronization action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentSyncAction {
    pub kind: AttachmentSyncActionKind,
    pub source: SectionIndexSource,
    pub section_title: String,
    pub path: String,
    pub absolute_path: Option<String>,
    pub message: String,
}

/// Stable synchronization action kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentSyncActionKind {
    MissingDirectory,
    MissingLinkedFile,
    OrphanFile,
    EmptyDirectory,
    StaleAttachTag,
}

impl AttachmentSyncActionKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingDirectory => "missingDirectory",
            Self::MissingLinkedFile => "missingLinkedFile",
            Self::OrphanFile => "orphanFile",
            Self::EmptyDirectory => "emptyDirectory",
            Self::StaleAttachTag => "staleAttachTag",
        }
    }
}

/// Caller-supplied `org-attach-archive-delete` policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentArchiveDeletePolicy {
    NotConfigured,
    Never,
    Query,
    Always,
}

impl AttachmentArchiveDeletePolicy {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotConfigured => "notConfigured",
            Self::Never => "never",
            Self::Query => "query",
            Self::Always => "always",
        }
    }
}

/// Stable attachment inventory entry kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachmentInventoryEntryKind {
    Directory { source: AttachmentDirectorySource },
    Link { link: AttachmentLink },
}

impl AttachmentInventoryEntryKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Directory { .. } => "directory",
            Self::Link { .. } => "link",
        }
    }
}

/// Git/VCS evidence for an attachment path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentVcsEvidence {
    pub status: AttachmentVcsStatus,
    pub annex: AttachmentAnnexEvidence,
    pub raw: Option<String>,
}

impl Default for AttachmentVcsEvidence {
    fn default() -> Self {
        Self {
            status: AttachmentVcsStatus::NotChecked,
            annex: AttachmentAnnexEvidence::default(),
            raw: None,
        }
    }
}

/// Stable VCS status category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentVcsStatus {
    NotChecked,
    Clean,
    Modified,
    Untracked,
    Missing,
    NotInGitWorktree,
    GitUnavailable,
}

impl AttachmentVcsStatus {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotChecked => "notChecked",
            Self::Clean => "clean",
            Self::Modified => "modified",
            Self::Untracked => "untracked",
            Self::Missing => "missing",
            Self::NotInGitWorktree => "notInGitWorktree",
            Self::GitUnavailable => "gitUnavailable",
        }
    }
}

/// Optional git-annex content-location evidence for an attachment path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentAnnexEvidence {
    pub status: AttachmentAnnexStatus,
    pub raw: Option<String>,
}

impl Default for AttachmentAnnexEvidence {
    fn default() -> Self {
        Self {
            status: AttachmentAnnexStatus::NotChecked,
            raw: None,
        }
    }
}

/// Stable git-annex status category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentAnnexStatus {
    NotChecked,
    NotAnnexRepository,
    GitAnnexUnavailable,
    Present,
    Missing,
    Unknown,
}

impl AttachmentAnnexStatus {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotChecked => "notChecked",
            Self::NotAnnexRepository => "notAnnexRepository",
            Self::GitAnnexUnavailable => "gitAnnexUnavailable",
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Unknown => "unknown",
        }
    }
}

/// Non-fatal attachment inventory warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInventoryWarning {
    pub kind: AttachmentInventoryWarningKind,
    pub message: String,
}

/// Stable inventory warning kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentInventoryWarningKind {
    MissingDirectory,
    MissingPath,
}

impl AttachmentInventoryWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingDirectory => "missingDirectory",
            Self::MissingPath => "missingPath",
        }
    }
}
