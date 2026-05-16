//! Property inheritance and allowed-value profile for agent-facing projections.

use super::SectionIndexSource;

/// Document-local property profile derived from native Org property metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyProfile {
    pub inheritance: PropertyInheritancePolicy,
    pub inherited_keys: Vec<String>,
    pub allowed_values: Vec<PropertyAllowedValueRecord>,
}

/// Caller-visible inheritance policy for projected effective properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyInheritancePolicy {
    None,
    All,
    Selective(Vec<String>),
    Pattern(String),
}

impl PropertyInheritancePolicy {
    /// Stable label for DTO and compact consumers.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::All => "all",
            Self::Selective(_) => "selective",
            Self::Pattern(_) => "pattern",
        }
    }
}

/// One `PROPERTY_ALL` descriptor visible to a document or section subtree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyAllowedValueRecord {
    pub source: Option<SectionIndexSource>,
    pub scope: PropertyAllowedValueScope,
    pub property: String,
    pub descriptor_key: String,
    pub values: Vec<String>,
}

/// Scope where an allowed-value descriptor was defined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyAllowedValueScope {
    FixedGlobal,
    Document,
    Section {
        outline_path: Vec<String>,
        level: usize,
        title: String,
    },
}

impl PropertyAllowedValueScope {
    /// Stable label for DTO and compact consumers.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FixedGlobal => "fixedGlobal",
            Self::Document => "document",
            Self::Section { .. } => "section",
        }
    }
}
