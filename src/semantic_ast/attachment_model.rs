//! Attachment metadata projected from ordinary Org properties and links.

/// Attachment metadata visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentState<A = ()> {
    pub has_attach_tag: bool,
    pub directory: Option<AttachmentDirectory<A>>,
}

impl<A> Default for AttachmentState<A> {
    fn default() -> Self {
        Self {
            has_attach_tag: false,
            directory: None,
        }
    }
}

impl<A> AttachmentState<A> {
    pub(crate) fn map_ann_with<B, F>(&self, f: &mut F) -> AttachmentState<B>
    where
        F: FnMut(&A) -> B,
    {
        AttachmentState {
            has_attach_tag: self.has_attach_tag,
            directory: self
                .directory
                .as_ref()
                .map(|directory| directory.map_ann_with(f)),
        }
    }

    pub(crate) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<AttachmentState<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(AttachmentState {
            has_attach_tag: self.has_attach_tag,
            directory: self
                .directory
                .as_ref()
                .map(|directory| directory.try_map_ann_with(f))
                .transpose()?,
        })
    }
}

/// Effective attachment directory for one section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentDirectory<A = ()> {
    pub ann: A,
    pub source: AttachmentDirectorySource,
    pub path: String,
}

impl<A> AttachmentDirectory<A> {
    pub(crate) fn from_property_parts(ann: A, key: &str, value: &str) -> Option<Self> {
        let source = AttachmentDirectorySource::from_property_key(key)?;
        let path = value.trim();
        (!path.is_empty()).then(|| Self {
            ann,
            source,
            path: path.to_string(),
        })
    }

    pub(crate) fn from_id_parts(ann: A, id: &str) -> Option<Self> {
        let id = id.trim();
        let (layout, path) = attachment_id_path(id)?;
        Some(Self {
            ann,
            source: AttachmentDirectorySource::IdDerived {
                id: id.to_string(),
                layout,
            },
            path: format!("data/{path}"),
        })
    }

    pub(crate) fn map_ann_with<B, F>(&self, f: &mut F) -> AttachmentDirectory<B>
    where
        F: FnMut(&A) -> B,
    {
        AttachmentDirectory {
            ann: f(&self.ann),
            source: self.source.clone(),
            path: self.path.clone(),
        }
    }

    pub(crate) fn try_map_ann_with<B, E, F>(&self, f: &mut F) -> Result<AttachmentDirectory<B>, E>
    where
        F: FnMut(&A) -> Result<B, E>,
    {
        Ok(AttachmentDirectory {
            ann: f(&self.ann)?,
            source: self.source.clone(),
            path: self.path.clone(),
        })
    }
}

/// Source that defines an attachment directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachmentDirectorySource {
    DirProperty,
    LegacyAttachDirProperty,
    IdDerived {
        id: String,
        layout: AttachmentIdPathLayout,
    },
}

impl AttachmentDirectorySource {
    fn from_property_key(key: &str) -> Option<Self> {
        if key.eq_ignore_ascii_case("DIR") {
            Some(Self::DirProperty)
        } else if key.eq_ignore_ascii_case("ATTACH_DIR") {
            Some(Self::LegacyAttachDirProperty)
        } else {
            None
        }
    }
}

/// Built-in Org attachment ID path layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentIdPathLayout {
    Uuid,
    Timestamp,
    Fallback,
}

/// File-like metadata for an `attachment:` link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentLink {
    pub path: String,
    pub search: Option<AttachmentLinkSearch>,
}

/// Search suffix attached to an `attachment:` link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentLinkSearch {
    pub raw: String,
    pub kind: AttachmentLinkSearchKind,
}

/// Normalized category for an attachment link search suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentLinkSearchKind {
    Headline,
    LineNumber,
    CustomId,
    Regexp,
    Text,
}

pub(crate) fn attachment_link_from_path(path: &str) -> Option<AttachmentLink> {
    let (protocol, target) = path.split_once(':')?;
    if !protocol.eq_ignore_ascii_case("attachment") {
        return None;
    }
    let (path, search) = target
        .split_once("::")
        .map(|(file_path, search)| (file_path.to_string(), attachment_link_search(search)))
        .unwrap_or_else(|| (target.to_string(), None));
    Some(AttachmentLink { path, search })
}

fn attachment_link_search(search: &str) -> Option<AttachmentLinkSearch> {
    let kind = if search.starts_with('*') {
        AttachmentLinkSearchKind::Headline
    } else if search.starts_with('#') {
        AttachmentLinkSearchKind::CustomId
    } else if search.starts_with('/') && search.ends_with('/') && search.len() > 1 {
        AttachmentLinkSearchKind::Regexp
    } else if search.chars().all(|ch| ch.is_ascii_digit()) {
        AttachmentLinkSearchKind::LineNumber
    } else {
        AttachmentLinkSearchKind::Text
    };
    Some(AttachmentLinkSearch {
        raw: search.to_string(),
        kind,
    })
}

fn attachment_id_path(id: &str) -> Option<(AttachmentIdPathLayout, String)> {
    if char_count(id) > 2 {
        let (prefix, suffix) = split_after_chars(id, 2)?;
        return Some((AttachmentIdPathLayout::Uuid, format!("{prefix}/{suffix}")));
    }
    if char_count(id) > 6 {
        let (prefix, suffix) = split_after_chars(id, 6)?;
        return Some((
            AttachmentIdPathLayout::Timestamp,
            format!("{prefix}/{suffix}"),
        ));
    }
    let (prefix, _) = split_after_chars(id, 1)?;
    Some((
        AttachmentIdPathLayout::Fallback,
        format!("__/{prefix}/{id}"),
    ))
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn split_after_chars(value: &str, count: usize) -> Option<(&str, &str)> {
    if count == 0 {
        return Some(("", value));
    }
    let index = value
        .char_indices()
        .nth(count)
        .map(|(index, _)| index)
        .or_else(|| (char_count(value) == count).then_some(value.len()))?;
    Some(value.split_at(index))
}
