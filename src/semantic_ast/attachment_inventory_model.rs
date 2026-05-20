//! Filesystem-aware attachment inventory DTOs.

use super::{AttachmentDirectorySource, AttachmentLink, SectionIndexSource};

/// Opt-in attachment inventory options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInventoryOptions {
    pub base_dir: String,
    pub check_vcs: bool,
    pub check_annex: bool,
    pub archive_delete_policy: AttachmentArchiveDeletePolicy,
}

impl AttachmentInventoryOptions {
    /// Creates inventory options rooted at a filesystem directory.
    pub fn new(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
            check_vcs: false,
            check_annex: false,
            archive_delete_policy: AttachmentArchiveDeletePolicy::NotConfigured,
        }
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
}

/// Filesystem-aware attachment inventory for one parsed document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttachmentInventory {
    pub entries: Vec<AttachmentInventoryEntry>,
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
