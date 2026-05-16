//! Link protocol registry records for non-executing Org projections.

use super::{LinkSearch, ParsedAnnotation};

/// One source-grounded Org link protocol or `#+LINK` abbreviation occurrence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkProtocolRecord {
    pub ann: ParsedAnnotation,
    pub source: LinkProtocolSource,
    pub protocol: String,
    pub kind: LinkProtocolKind,
    pub raw: String,
    pub target: String,
    pub search: Option<LinkSearch>,
    pub replacement: Option<String>,
    pub org_protocol: Option<OrgProtocolCall>,
}

/// Where a protocol registry record came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkProtocolSource {
    /// An inline, bracket, angle, or plain link occurrence.
    Link,
    /// A `#+LINK:` abbreviation definition.
    AbbreviationDefinition,
}

/// Stable protocol family for parser, lint, and frontend consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkProtocolKind {
    File,
    Attachment,
    InternalId,
    CodeReference,
    Web,
    Message,
    Documentation,
    Executable,
    OrgProtocol,
    Abbreviation,
    Custom,
}

impl LinkProtocolKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Attachment => "attachment",
            Self::InternalId => "internalId",
            Self::CodeReference => "codeReference",
            Self::Web => "web",
            Self::Message => "message",
            Self::Documentation => "documentation",
            Self::Executable => "executable",
            Self::OrgProtocol => "orgProtocol",
            Self::Abbreviation => "abbreviation",
            Self::Custom => "custom",
        }
    }
}

/// Parsed, inert metadata for an `org-protocol:` link occurrence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgProtocolCall {
    pub subprotocol: String,
    pub kind: OrgProtocolKind,
    pub parameters: Vec<OrgProtocolParameter>,
}

/// Known default org-protocol handler family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgProtocolKind {
    StoreLink,
    Capture,
    OpenSource,
    Custom,
}

impl OrgProtocolKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StoreLink => "storeLink",
            Self::Capture => "capture",
            Self::OpenSource => "openSource",
            Self::Custom => "custom",
        }
    }
}

/// One query-style parameter from an org-protocol URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgProtocolParameter {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
}
