//! Filesystem-aware attachment inventory DTOs.

use super::{AttachmentDirectorySource, AttachmentLink, SectionIndexSource};

/// Opt-in attachment inventory options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInventoryOptions {
    pub base_dir: String,
    pub check_vcs: bool,
}

impl AttachmentInventoryOptions {
    /// Creates inventory options rooted at a filesystem directory.
    pub fn new(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
            check_vcs: false,
        }
    }

    /// Enables or disables git status checks for discovered paths.
    pub fn check_vcs(mut self, check_vcs: bool) -> Self {
        self.check_vcs = check_vcs;
        self
    }
}

/// Filesystem-aware attachment inventory for one parsed document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttachmentInventory {
    pub entries: Vec<AttachmentInventoryEntry>,
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
    pub raw: Option<String>,
}

impl Default for AttachmentVcsEvidence {
    fn default() -> Self {
        Self {
            status: AttachmentVcsStatus::NotChecked,
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

/// Non-fatal attachment inventory warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentInventoryWarning {
    pub kind: AttachmentInventoryWarningKind,
    pub message: String,
}

/// Stable inventory warning kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentInventoryWarningKind {
    MissingPath,
}

impl AttachmentInventoryWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingPath => "missingPath",
        }
    }
}
