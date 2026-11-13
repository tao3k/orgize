//! Selector model for compact Org element queries.

use std::fmt;

use super::elements_bridge_model::{OrgElementsIndexCategory, OrgElementsIndexKind};
use super::elements_bridge_query::OrgElementsIndexQuery;

/// Org-mode-style selector for element records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementSelector {
    pub element_type: OrgElementsIndexKind,
    pub name: Option<String>,
    pub language: Option<String>,
}

impl OrgElementSelector {
    pub fn new(element_type: impl Into<OrgElementsIndexKind>) -> Self {
        Self {
            element_type: element_type.into(),
            name: None,
            language: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn parse_plist(input: &str) -> Result<Self, OrgElementSelectorParseError> {
        let properties = super::org_elements_query_expr::selector_plist_properties(input)?;
        Self::from_native_properties(properties)
    }

    pub(crate) fn from_native_properties(
        properties: Vec<(String, String)>,
    ) -> Result<Self, OrgElementSelectorParseError> {
        let mut element_type = None;
        let mut name = None;
        let mut language = None;
        for (key, value) in properties {
            match key.as_str() {
                ":type" => element_type = Some(OrgElementsIndexKind::new(value)),
                ":name" => name = Some(value),
                ":language" => language = Some(value),
                _ => return Err(OrgElementSelectorParseError::UnknownKey(key)),
            }
        }

        let element_type = element_type.ok_or(OrgElementSelectorParseError::MissingType)?;
        Ok(Self {
            element_type,
            name,
            language,
        })
    }

    pub fn to_index_query(&self) -> OrgElementsIndexQuery {
        let mut query = OrgElementsIndexQuery::new()
            .category(OrgElementsIndexCategory::Element)
            .kind(self.element_type.clone());
        if let Some(name) = &self.name {
            query = query.affiliated_name(name.clone());
        }
        if let Some(language) = &self.language {
            query = query.summary_eq("language", language.clone());
        }
        query
    }
}

/// Parse error for a compact Org element selector plist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgElementSelectorParseError {
    InvalidShape,
    OddPropertyList,
    UnterminatedString,
    MissingType,
    UnknownKey(String),
}

impl fmt::Display for OrgElementSelectorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidShape => {
                write!(f, "selector must use `(:org-element (:type ...))`")
            }
            Self::OddPropertyList => {
                write!(f, "selector property list must contain key/value pairs")
            }
            Self::UnterminatedString => write!(f, "selector contains an unterminated string"),
            Self::MissingType => write!(f, "selector must include :type"),
            Self::UnknownKey(key) => write!(f, "selector contains unsupported key `{key}`"),
        }
    }
}

impl std::error::Error for OrgElementSelectorParseError {}
