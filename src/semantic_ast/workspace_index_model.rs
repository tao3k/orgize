//! Workspace-level semantic index records.

use super::attachment_model::AttachmentLink;
use super::link_model::{FileLink, LinkSearch};
use super::model::TargetKind;
use super::section_index_model::{SectionIndexRecord, SectionIndexSource};

/// Cross-document index built from semantic Org projections.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceIndex {
    pub documents: Vec<WorkspaceDocument>,
    pub targets: Vec<WorkspaceTargetRef>,
    pub links: Vec<WorkspaceLinkRef>,
    pub attachments: Vec<WorkspaceAttachmentRef>,
    pub issues: Vec<WorkspaceIssue>,
}

/// One parsed source file included in a workspace index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceDocument {
    pub source_file: String,
    pub summary: WorkspaceDocumentSummary,
    pub sections: Vec<SectionIndexRecord>,
}

/// Compact per-document counts for indexers and UI summaries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceDocumentSummary {
    pub section_count: usize,
    pub target_count: usize,
    pub link_count: usize,
    pub attachment_section_count: usize,
    pub attachment_link_count: usize,
    pub source_block_count: usize,
}

/// One target that can resolve an Org internal link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTargetRef {
    pub source_file: String,
    pub source: SectionIndexSource,
    pub section_title: String,
    pub outline_path: Vec<String>,
    pub kind: TargetKind,
    pub key: String,
    pub value: String,
}

/// One Org link visible from a workspace source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceLinkRef {
    pub source_file: String,
    pub source: SectionIndexSource,
    pub section_title: String,
    pub outline_path: Vec<String>,
    pub path: String,
    pub description: String,
    pub search: Option<LinkSearch>,
    pub attachment: Option<AttachmentLink>,
    pub file: Option<FileLink>,
    pub resolved_target: Option<WorkspaceResolvedTarget>,
}

/// A target selected by workspace link resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceResolvedTarget {
    pub source_file: String,
    pub section_title: String,
    pub outline_path: Vec<String>,
    pub kind: TargetKind,
    pub key: String,
    pub value: String,
}

/// Attachment evidence collected across the workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceAttachmentRef {
    pub source_file: String,
    pub source: SectionIndexSource,
    pub section_title: String,
    pub outline_path: Vec<String>,
    pub kind: WorkspaceAttachmentKind,
    pub path: String,
    pub link: Option<AttachmentLink>,
}

/// Attachment evidence source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceAttachmentKind {
    /// Section-level attachment directory from `ID`, `DIR`, or `ATTACH_DIR`.
    SectionDirectory,
    /// Ordinary `attachment:` link.
    Link,
}

/// Cross-document indexing issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceIssue {
    pub source_file: String,
    pub source: SectionIndexSource,
    pub kind: WorkspaceIssueKind,
    pub message: String,
}

/// Stable issue categories emitted by workspace indexing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceIssueKind {
    DuplicateId { key: String },
    DuplicateCustomId { key: String },
    AmbiguousInternalLink { key: String },
    UnresolvedIdLink { key: String },
    UnresolvedCustomIdLink { key: String },
    UnresolvedFootnoteLink { key: String },
    UnresolvedCodeRefLink { key: String },
}
