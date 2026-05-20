//! Document-local section records for downstream index builders.

use super::attachment_model::{AttachmentDirectorySource, AttachmentLink};
use super::lifecycle_model::LifecycleRecordKind;
use super::link_model::{FileLink, LinkSearch};
use super::model::{ParsedAnnotation, Planning, SourcePosition, TargetKind, TodoKeyword};
use super::property_model::Priority;

/// One source-grounded Org section projected for downstream indexing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexRecord {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub outline_path_text: Vec<String>,
    pub level: usize,
    pub title: String,
    pub title_text: String,
    pub body: Vec<SectionIndexTextSlice>,
    pub todo: Option<TodoKeyword>,
    pub priority: Priority,
    pub category: Option<SectionIndexCategory>,
    pub tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub properties: Vec<SectionIndexProperty>,
    pub effective_properties: Vec<SectionIndexProperty>,
    pub special_properties: Vec<SectionIndexSpecialProperty>,
    pub planning: Planning,
    pub is_comment: bool,
    pub archive: SectionIndexArchive,
    pub attachment: SectionIndexAttachment,
    pub links: Vec<SectionIndexLink>,
    pub targets: Vec<SectionIndexTarget>,
    pub lifecycle: Vec<SectionIndexLifecycleRecord>,
}

/// Source location for section index records and nested evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexSource {
    pub start: SourcePosition,
    pub end: SourcePosition,
    pub range_start: u32,
    pub range_end: u32,
}

impl SectionIndexSource {
    pub(crate) fn from_annotation(annotation: &ParsedAnnotation) -> Self {
        Self {
            start: annotation.start,
            end: annotation.end,
            range_start: annotation.range.start().into(),
            range_end: annotation.range.end().into(),
        }
    }
}

/// A source-backed text slice from the direct body of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexTextSlice {
    pub source: SectionIndexSource,
    pub text: String,
}

/// A local or effective Org property visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexProperty {
    pub source: SectionIndexSource,
    pub key: String,
    pub value: String,
}

/// Official Org special property projected for a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexSpecialProperty {
    pub source: SectionIndexSource,
    pub name: String,
    pub value: String,
}

/// Section category projected for agenda/search records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexCategory(String);

impl SectionIndexCategory {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the category text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Archive state visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexArchive {
    pub archived: bool,
    pub has_archive_tag: bool,
    pub location: Option<String>,
}

/// Attachment state visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexAttachment {
    pub has_attach_tag: bool,
    pub directory: Option<SectionIndexAttachmentDirectory>,
}

/// Attachment directory evidence visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexAttachmentDirectory {
    pub source: AttachmentDirectorySource,
    pub path: String,
}

/// Link metadata visible from a section title or direct body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexLink {
    pub source: SectionIndexSource,
    pub path: String,
    pub description: String,
    pub search: Option<LinkSearch>,
    pub attachment: Option<AttachmentLink>,
    pub file: Option<FileLink>,
}

/// Document-local target metadata visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexTarget {
    pub source: SectionIndexSource,
    pub kind: TargetKind,
    pub key: String,
    pub value: String,
}

/// Lifecycle event evidence visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionIndexLifecycleRecord {
    pub source: SectionIndexSource,
    pub kind: LifecycleRecordKind,
    pub raw: String,
}
