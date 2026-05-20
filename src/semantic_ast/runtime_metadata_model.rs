//! Source-backed runtime-adjacent Org metadata.

use super::section_index_model::SectionIndexSource;

/// Non-executing metadata plan for runtime-adjacent Org features.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMetadataPlan {
    pub feeds: Vec<FeedStatusRecord>,
    pub timers: Vec<TimerRecord>,
    pub mobile: MobileSyncMetadata,
    pub boundaries: Vec<RuntimeMetadataBoundary>,
    pub warnings: Vec<RuntimeMetadataWarning>,
}

/// `org-feed.el` status drawer evidence kept in an Org file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedStatusRecord {
    pub source: SectionIndexSource,
    pub section_title: String,
    pub drawer: FeedStatusDrawerName,
    pub raw: String,
    pub entry_count: usize,
    pub readable: bool,
}

/// Feed status drawer name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedStatusDrawerName(String);

impl FeedStatusDrawerName {
    /// Creates a drawer name from parsed source text.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the drawer name text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Relative/countdown timer stamp found in ordinary Org content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimerRecord {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub context: TimerContext,
    pub raw: String,
    pub total_seconds: i64,
}

/// Context that contained a timer stamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerContext {
    Headline,
    Paragraph,
    ListItemTag,
}

impl TimerContext {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Headline => "headline",
            Self::Paragraph => "paragraph",
            Self::ListItemTag => "listItemTag",
        }
    }
}

/// MobileOrg-compatible metadata visible in source.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MobileSyncMetadata {
    pub readonly: Vec<MobileReadonlyKeyword>,
    pub all_priorities: Vec<MobilePriorityDeclaration>,
    pub index_links: Vec<MobileIndexLink>,
    pub flagged_sections: Vec<MobileFlaggedSection>,
    pub original_ids: Vec<MobileOriginalId>,
}

/// `#+READONLY` marker from a MobileOrg index-style file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileReadonlyKeyword {
    pub source: SectionIndexSource,
    pub value: String,
}

/// `#+ALLPRIORITIES:` marker emitted by Org Mobile index files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobilePriorityDeclaration {
    pub source: SectionIndexSource,
    pub values: Vec<String>,
    pub raw: String,
}

/// File link exposed from a MobileOrg index-style heading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileIndexLink {
    pub source: SectionIndexSource,
    pub title: String,
    pub file: String,
    pub description: String,
}

/// Section carrying the MobileOrg `FLAGGED` intervention tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileFlaggedSection {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub title: String,
    pub original_id: Option<String>,
    pub mobile_properties: Vec<MobileProperty>,
}

/// `ORIGINAL_ID` evidence used by generated mobile agenda entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileOriginalId {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub title: String,
    pub value: String,
}

/// Mobile-related property retained from a section property drawer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileProperty {
    pub source: SectionIndexSource,
    pub key: String,
    pub value: String,
}

/// Runtime behavior that remains intentionally outside parser core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMetadataBoundary {
    pub kind: RuntimeMetadataBoundaryKind,
    pub message: String,
}

/// Stable runtime boundary category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMetadataBoundaryKind {
    FeedNetworkUpdate,
    TimerRuntimeState,
    MobileFilesystemSync,
    OrgPersistCache,
}

impl RuntimeMetadataBoundaryKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FeedNetworkUpdate => "feedNetworkUpdate",
            Self::TimerRuntimeState => "timerRuntimeState",
            Self::MobileFilesystemSync => "mobileFilesystemSync",
            Self::OrgPersistCache => "orgPersistCache",
        }
    }
}

/// Non-fatal warning produced while collecting runtime metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMetadataWarning {
    pub kind: RuntimeMetadataWarningKind,
    pub message: String,
}

/// Stable runtime metadata warning category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMetadataWarningKind {
    UnreadableFeedStatus,
    MobileReadonlyWithoutIndexLinks,
}

impl RuntimeMetadataWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnreadableFeedStatus => "unreadableFeedStatus",
            Self::MobileReadonlyWithoutIndexLinks => "mobileReadonlyWithoutIndexLinks",
        }
    }
}
