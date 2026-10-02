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
    AttachDirProperty,
    IdDerived {
        id: String,
        layout: AttachmentIdPathLayout,
    },
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
