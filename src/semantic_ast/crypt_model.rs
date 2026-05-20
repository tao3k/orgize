//! Org Crypt projection types.

use super::section_index_model::SectionIndexSource;

/// Source-grounded Org Crypt state for one section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptState {
    pub source: SectionIndexSource,
    pub outline_path: Vec<String>,
    pub level: usize,
    pub title: String,
    pub tag: CryptTag,
    pub has_direct_tag: bool,
    pub has_inherited_tag: bool,
    pub crypt_key: Option<CryptKey>,
    pub encrypted_payload: bool,
    pub body_is_opaque: bool,
    pub warnings: Vec<CryptWarning>,
}

impl CryptState {
    /// Returns true when this state marks a subtree body as opaque for
    /// indexing/export consumers.
    pub fn marks_opaque_body(&self) -> bool {
        self.body_is_opaque
    }
}

/// Org Crypt tag name recognized by this projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptTag(String);

impl CryptTag {
    /// Creates the default Org Crypt tag marker.
    pub fn default_org_crypt() -> Self {
        Self("crypt".to_string())
    }

    /// Returns the tag text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// `CRYPTKEY` evidence visible from a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptKey {
    pub source: SectionIndexSource,
    pub value: String,
    pub inherited: bool,
}

/// Non-fatal Org Crypt alignment warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptWarning {
    pub kind: CryptWarningKind,
    pub message: String,
}

/// Stable crypt warning category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CryptWarningKind {
    InheritedCryptTag,
    PlaintextCryptBody,
    CryptKeyWithoutCryptTag,
}

impl CryptWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InheritedCryptTag => "inheritedCryptTag",
            Self::PlaintextCryptBody => "plaintextCryptBody",
            Self::CryptKeyWithoutCryptTag => "cryptKeyWithoutCryptTag",
        }
    }
}
