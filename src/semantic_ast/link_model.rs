//! Link semantic data model.

/// Normalized link target classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkTarget {
    /// URI-like link target split into protocol and path.
    Uri { protocol: String, path: String },
    /// Internal target such as `#custom-id`.
    Internal(String),
    /// Link target without a dedicated semantic classifier yet.
    Unresolved(String),
}

/// Original link path text as it appears in the link target position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkPath(String);

impl LinkPath {
    /// Creates a link path from parser-owned text.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the path and returns its owned text.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for LinkPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for LinkPath {
    fn from(path: String) -> Self {
        Self::new(path)
    }
}

impl From<&str> for LinkPath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl AsRef<str> for LinkPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Whether a link had an explicit Org description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkDescriptionState {
    /// Link has no explicit description.
    None,
    /// Link was written with a description part.
    Explicit,
}

impl LinkDescriptionState {
    /// Returns true when the source link had an explicit description.
    pub const fn has_description(self) -> bool {
        matches!(self, Self::Explicit)
    }
}

/// Media classification for link exporter behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkMediaKind {
    /// Normal link.
    Normal,
    /// Image link.
    Image,
}

impl LinkMediaKind {
    /// Returns true when the link should be treated as an image.
    pub const fn is_image(self) -> bool {
        matches!(self, Self::Image)
    }
}

/// Search suffix attached to an internal link target, such as `id:x::*Heading`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSearch {
    pub raw: String,
    pub kind: LinkSearchKind,
    pub normalized: String,
}

/// Normalized category for an Org link search suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkSearchKind {
    Headline,
    LineNumber,
    CustomId,
    Regexp,
    Text,
}

/// File-like metadata for ordinary Org `file:` links.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileLink {
    pub protocol: String,
    pub path: String,
    pub path_kind: FileLinkPathKind,
    pub search: Option<LinkSearch>,
}

/// Normalized local/remote shape for an Org file link path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileLinkPathKind {
    Empty,
    Absolute,
    HomeRelative,
    Relative,
    Remote,
}
