//! Host-loaded property schema contracts for Org property drawers.

use super::SectionIndexSource;

/// Ordinary Org property used to reference a loaded property schema contract.
pub const PROPERTY_SCHEMA_PROPERTY: &str = "PROPERTY_SCHEMA";

/// User or host loaded property schema registry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PropertySchemaRegistry {
    pub contracts: Vec<PropertySchemaContract>,
}

impl PropertySchemaRegistry {
    pub fn new(contracts: impl IntoIterator<Item = PropertySchemaContract>) -> Self {
        Self {
            contracts: contracts.into_iter().collect(),
        }
    }

    pub(crate) fn resolve(
        &self,
        reference: &PropertySchemaReference,
    ) -> Option<&PropertySchemaContract> {
        self.contracts.iter().find(|contract| {
            reference_matches(contract.id.as_str(), reference)
                || contract
                    .aliases
                    .iter()
                    .any(|alias| reference_matches(alias.as_str(), reference))
        })
    }
}

/// One loaded schema contract for property drawers that reference it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySchemaContract {
    pub id: String,
    pub aliases: Vec<String>,
    pub fields: Vec<PropertySchemaField>,
    pub allow_unknown_properties: bool,
}

impl PropertySchemaContract {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            aliases: Vec::new(),
            fields: Vec::new(),
            allow_unknown_properties: true,
        }
    }

    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn field(mut self, field: PropertySchemaField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn allow_unknown_properties(mut self, allow_unknown_properties: bool) -> Self {
        self.allow_unknown_properties = allow_unknown_properties;
        self
    }

    pub(crate) fn field_for(&self, key: &str) -> Option<&PropertySchemaField> {
        self.fields
            .iter()
            .find(|field| field.key.eq_ignore_ascii_case(key))
    }
}

/// Field rule inside a loaded property schema contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySchemaField {
    pub key: String,
    pub required: bool,
    pub value_rule: PropertySchemaValueRule,
}

impl PropertySchemaField {
    pub fn optional(key: impl Into<String>, value_rule: PropertySchemaValueRule) -> Self {
        Self {
            key: key.into(),
            required: false,
            value_rule,
        }
    }

    pub fn required(key: impl Into<String>, value_rule: PropertySchemaValueRule) -> Self {
        Self {
            key: key.into(),
            required: true,
            value_rule,
        }
    }
}

/// Value rule for one property schema field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertySchemaValueRule {
    Any,
    NonEmpty,
    OneOf(Vec<String>),
}

impl PropertySchemaValueRule {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::NonEmpty => "nonEmpty",
            Self::OneOf(_) => "oneOf",
        }
    }
}

/// One local drawer that referenced a property schema contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySchemaApplication {
    pub source: SectionIndexSource,
    pub scope: PropertySchemaScope,
    pub reference: PropertySchemaReference,
    pub contract_id: Option<String>,
    pub findings: Vec<PropertySchemaFinding>,
}

/// Drawer scope for a property schema reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertySchemaScope {
    Document,
    Section {
        outline_path: Vec<String>,
        level: usize,
        title: String,
    },
}

impl PropertySchemaScope {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Section { .. } => "section",
        }
    }
}

/// Parsed reference value from a `PROPERTY_SCHEMA` property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySchemaReference {
    pub raw: String,
    pub normalized: String,
    pub kind: PropertySchemaReferenceKind,
}

/// Reference syntax used by `PROPERTY_SCHEMA`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertySchemaReferenceKind {
    Empty,
    ContractId,
    File,
    OrgFileLink,
    Macro,
}

impl PropertySchemaReferenceKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::ContractId => "contractId",
            Self::File => "file",
            Self::OrgFileLink => "orgFileLink",
            Self::Macro => "macro",
        }
    }
}

/// One schema contract validation finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertySchemaFinding {
    pub source: SectionIndexSource,
    pub kind: PropertySchemaFindingKind,
    pub property: Option<String>,
    pub actual: Option<String>,
    pub expected: Vec<String>,
    pub message: String,
}

/// Stable property schema finding categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertySchemaFindingKind {
    EmptyReference,
    UnresolvedReference,
    MissingRequiredProperty,
    UnknownProperty,
    EmptyValue,
    DisallowedValue,
}

impl PropertySchemaFindingKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyReference => "emptyReference",
            Self::UnresolvedReference => "unresolvedReference",
            Self::MissingRequiredProperty => "missingRequiredProperty",
            Self::UnknownProperty => "unknownProperty",
            Self::EmptyValue => "emptyValue",
            Self::DisallowedValue => "disallowedValue",
        }
    }
}

fn reference_matches(candidate: &str, reference: &PropertySchemaReference) -> bool {
    let candidate = candidate.trim();
    !candidate.is_empty()
        && (candidate == reference.raw.trim() || candidate == reference.normalized.trim())
}
