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
        let tokens = tokenize_selector_plist(input)?;
        if tokens.len() < 6
            || tokens.first().map(String::as_str) != Some("(")
            || tokens.get(1).map(String::as_str) != Some(":org-element")
            || tokens.get(2).map(String::as_str) != Some("(")
            || tokens
                .get(tokens.len().saturating_sub(2))
                .map(String::as_str)
                != Some(")")
            || tokens.last().map(String::as_str) != Some(")")
        {
            return Err(OrgElementSelectorParseError::InvalidShape);
        }
        let properties = &tokens[3..tokens.len().saturating_sub(2)];
        if properties.len() % 2 != 0 {
            return Err(OrgElementSelectorParseError::OddPropertyList);
        }

        let mut element_type = None;
        let mut name = None;
        let mut language = None;
        for pair in properties.chunks(2) {
            let key = pair[0].as_str();
            let value = pair[1].clone();
            match key {
                ":type" => element_type = Some(OrgElementsIndexKind::new(value)),
                ":name" => name = Some(value),
                ":language" => language = Some(value),
                _ => return Err(OrgElementSelectorParseError::UnknownKey(pair[0].clone())),
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

fn tokenize_selector_plist(input: &str) -> Result<Vec<String>, OrgElementSelectorParseError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '(' | ')' => tokens.push(ch.to_string()),
            '"' => {
                let mut value = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => {
                            let Some(escaped) = chars.next() else {
                                return Err(OrgElementSelectorParseError::UnterminatedString);
                            };
                            value.push(escaped);
                        }
                        Some(next) => value.push(next),
                        None => return Err(OrgElementSelectorParseError::UnterminatedString),
                    }
                }
                tokens.push(value);
            }
            ch if ch.is_whitespace() => {}
            _ => {
                let mut value = String::from(ch);
                while let Some(next) = chars.peek().copied() {
                    if next.is_whitespace() || matches!(next, '(' | ')') {
                        break;
                    }
                    value.push(next);
                    chars.next();
                }
                tokens.push(value);
            }
        }
    }
    Ok(tokens)
}
