//! Lifecycle, LOGBOOK, and archive projection types.

use super::property_model::OrgDuration;

/// Archive destination collected from `#+ARCHIVE:` or `ARCHIVE` properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveLocation<A = ()> {
    pub ann: A,
    pub value: String,
    pub file: Option<String>,
    pub heading: Option<String>,
}

impl<A> ArchiveLocation<A> {
    pub(crate) fn from_value(ann: A, value: impl Into<String>) -> Self {
        let value = value.into();
        let trimmed = value.trim().to_string();
        let (file, heading) = trimmed
            .as_str()
            .split_once("::")
            .map(|(file, heading)| (text_or_none(file), text_or_none(heading)))
            .unwrap_or((text_or_none(trimmed.as_str()), None));
        Self {
            ann,
            value: trimmed,
            file,
            heading,
        }
    }

    /// Returns true when the archive destination has no usable target text.
    pub fn is_empty(&self) -> bool {
        self.value.trim().is_empty()
    }

    pub(crate) fn map_ann_with<B, F>(&self, f: &mut F) -> ArchiveLocation<B>
    where
        F: FnMut(&A) -> B,
    {
        ArchiveLocation {
            ann: f(&self.ann),
            value: self.value.clone(),
            file: self.file.clone(),
            heading: self.heading.clone(),
        }
    }

    pub(crate) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ArchiveLocation<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(ArchiveLocation {
            ann: f(&self.ann)?,
            value: self.value.clone(),
            file: self.file.clone(),
            heading: self.heading.clone(),
        })
    }
}

/// Effective archive metadata for one section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveState<A = ()> {
    pub archived: bool,
    pub has_archive_tag: bool,
    pub property_location: Option<ArchiveLocation<A>>,
    pub keyword_location: Option<ArchiveLocation<A>>,
}

impl<A> Default for ArchiveState<A> {
    fn default() -> Self {
        Self {
            archived: false,
            has_archive_tag: false,
            property_location: None,
            keyword_location: None,
        }
    }
}

impl<A> ArchiveState<A> {
    /// Returns the effective archive destination, preferring an `ARCHIVE`
    /// property over a document-level `#+ARCHIVE:` keyword.
    pub fn location(&self) -> Option<&ArchiveLocation<A>> {
        self.property_location
            .as_ref()
            .or(self.keyword_location.as_ref())
    }

    pub(crate) fn map_ann_with<B, F>(&self, f: &mut F) -> ArchiveState<B>
    where
        F: FnMut(&A) -> B,
    {
        ArchiveState {
            archived: self.archived,
            has_archive_tag: self.has_archive_tag,
            property_location: self
                .property_location
                .as_ref()
                .map(|location| location.map_ann_with(f)),
            keyword_location: self
                .keyword_location
                .as_ref()
                .map(|location| location.map_ann_with(f)),
        }
    }

    pub(crate) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<ArchiveState<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(ArchiveState {
            archived: self.archived,
            has_archive_tag: self.has_archive_tag,
            property_location: self
                .property_location
                .as_ref()
                .map(|location| location.try_map_ann_with(f))
                .transpose()?,
            keyword_location: self
                .keyword_location
                .as_ref()
                .map(|location| location.try_map_ann_with(f))
                .transpose()?,
        })
    }
}

/// One lifecycle event projected from a LOGBOOK-like drawer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleRecord<A = ()> {
    pub ann: A,
    pub section_anchor: Option<String>,
    pub section_title: String,
    pub kind: LifecycleRecordKind,
    pub raw: String,
}

/// Kind of lifecycle event recognized from ordinary Org LOGBOOK content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LifecycleRecordKind {
    StateChange {
        to: Option<String>,
        from: Option<String>,
        timestamp: Option<String>,
    },
    Note {
        timestamp: Option<String>,
    },
    Refile {
        target: Option<String>,
        timestamp: Option<String>,
    },
    Reschedule {
        from: Option<String>,
        to: Option<String>,
        timestamp: Option<String>,
    },
    Redeadline {
        from: Option<String>,
        to: Option<String>,
        timestamp: Option<String>,
    },
    Clock {
        duration: Option<OrgDuration>,
        timestamp: Option<String>,
    },
    MalformedLogbook {
        reason: String,
    },
}

impl LifecycleRecordKind {
    /// Human-readable lifecycle kind for compact projections and lint output.
    pub fn title(&self) -> &'static str {
        match self {
            Self::StateChange { .. } => "state change",
            Self::Note { .. } => "note",
            Self::Refile { .. } => "refile",
            Self::Reschedule { .. } => "reschedule",
            Self::Redeadline { .. } => "redeadline",
            Self::Clock { .. } => "clock",
            Self::MalformedLogbook { .. } => "malformed logbook",
        }
    }
}

fn text_or_none(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}
