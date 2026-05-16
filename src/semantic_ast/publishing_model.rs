//! Export and publishing side-table records.

use super::{IncludeDirective, KeywordAttribute};

/// Publishing-oriented settings projected from Org keywords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingSettings<A = ()> {
    pub export_file_name: Option<PublishingKeyword<A>>,
    pub setup_files: Vec<PublishingKeyword<A>>,
    pub binds: Vec<PublishingBind<A>>,
    pub options: Vec<PublishingOption<A>>,
    pub attributes: Vec<PublishingAttribute<A>>,
    pub backend_keywords: Vec<PublishingKeyword<A>>,
    pub includes: Vec<IncludeDirective<A>>,
}

impl<A> Default for PublishingSettings<A> {
    fn default() -> Self {
        Self {
            export_file_name: None,
            setup_files: Vec::new(),
            binds: Vec::new(),
            options: Vec::new(),
            attributes: Vec::new(),
            backend_keywords: Vec::new(),
            includes: Vec::new(),
        }
    }
}

/// A source-backed publishing keyword value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingKeyword<A = ()> {
    pub ann: A,
    pub key: String,
    pub value: String,
}

/// One `#+BIND:` assignment, retained as inert metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingBind<A = ()> {
    pub ann: A,
    pub name: String,
    pub value: String,
    pub raw: String,
}

/// One token from `#+OPTIONS:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingOption<A = ()> {
    pub ann: A,
    pub key: String,
    pub value: String,
    pub raw: String,
    pub kind: PublishingOptionKind,
}

/// High-usage export option categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishingOptionKind {
    HeadlineLevels,
    SectionNumbering,
    SpecialStrings,
    Entities,
    TodoKeywords,
    Tags,
    Timestamps,
    Author,
    Creator,
    Date,
    Email,
    Title,
    Drawers,
    Planning,
    Priorities,
    BrokenLinks,
    Other,
}

/// Backend-specific `#+ATTR_*` keyword metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingAttribute<A = ()> {
    pub ann: A,
    pub backend: String,
    pub optional: Option<String>,
    pub attributes: Vec<KeywordAttribute>,
    pub raw: String,
}
