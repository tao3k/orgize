//! Typed projection for Org-owned interactive choice windows.

use std::{error::Error, fmt};

use super::SourceBlockSource;

/// One validated interactive choice window declared by an Org source block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgInteractiveChoice {
    pub source: SourceBlockSource,
    pub id: String,
    pub method: String,
    pub stage: String,
    pub group: Option<String>,
    pub target: Option<String>,
    pub create: Option<String>,
    pub info: String,
    pub categories: Vec<OrgInteractiveCategory>,
    pub entries: Vec<OrgInteractiveChoiceEntry>,
}

/// One compact key-to-choice mapping from an interactive window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgInteractiveCategory {
    pub key: String,
    pub value: String,
    pub detail: bool,
}

/// One selectable row from an interactive choice details table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgInteractiveChoiceEntry {
    pub number: String,
    pub id: String,
    pub contract: Option<String>,
    pub full: String,
    pub use_if: String,
}

/// Validation error for an Org-Interactive source block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgInteractiveParseError {
    pub message: String,
}

impl OrgInteractiveParseError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for OrgInteractiveParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl Error for OrgInteractiveParseError {}
