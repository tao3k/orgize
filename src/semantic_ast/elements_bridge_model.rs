//! Model types for explicit Org element bindings.

use std::{collections::BTreeMap, fmt};

/// Explicit host execution directives projected from Org keywords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsExecutionPlan<A = ()> {
    pub python_directives: Vec<PythonDirective<A>>,
}

/// One flat, source-backed record in the Org elements index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsIndexRecord<A = ()> {
    pub ann: A,
    pub ordinal: usize,
    pub category: OrgElementsIndexCategory,
    pub kind: OrgElementsIndexKind,
    pub affiliated: OrgElementsAffiliatedProperties,
    pub outline_path: Vec<String>,
    pub context: String,
    pub summary: OrgElementsIndexSummary,
}

/// Org-mode-style properties derived from affiliated keywords on an element.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgElementsAffiliatedProperties {
    pub name: Option<String>,
}

/// Stable node kind label in the Org elements flat index.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrgElementsIndexKind(String);

impl OrgElementsIndexKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Predicate for selecting records from `Document::org_elements_index()`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgElementsIndexQuery {
    pub category: Option<OrgElementsIndexCategory>,
    pub kind: Option<OrgElementsIndexKind>,
    pub affiliated_name: Option<String>,
    pub context: Option<String>,
    pub outline_path_prefix: Vec<String>,
    pub summary_equals: Vec<OrgElementsIndexSummaryPredicate>,
    pub summary_contains: Vec<OrgElementsIndexSummaryTextPredicate>,
    pub limit: Option<usize>,
}

impl OrgElementsIndexQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn category(mut self, category: OrgElementsIndexCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn kind(mut self, kind: impl Into<OrgElementsIndexKind>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    pub fn affiliated_name(mut self, name: impl Into<String>) -> Self {
        self.affiliated_name = Some(name.into());
        self
    }

    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn outline_path_prefix(
        mut self,
        outline_path_prefix: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.outline_path_prefix = outline_path_prefix.into_iter().map(Into::into).collect();
        self
    }

    pub fn summary_eq(
        mut self,
        key: impl Into<String>,
        value: impl Into<OrgElementsIndexSummaryValue>,
    ) -> Self {
        self.summary_equals.push(OrgElementsIndexSummaryPredicate {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn summary_contains(mut self, key: impl Into<String>, needle: impl Into<String>) -> Self {
        self.summary_contains
            .push(OrgElementsIndexSummaryTextPredicate {
                key: key.into(),
                needle: needle.into(),
            });
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn matches<A>(&self, record: &OrgElementsIndexRecord<A>) -> bool {
        if let Some(category) = self.category
            && record.category != category
        {
            return false;
        }
        if let Some(kind) = &self.kind
            && record.kind != *kind
        {
            return false;
        }
        if let Some(name) = &self.affiliated_name
            && record.affiliated.name.as_ref() != Some(name)
        {
            return false;
        }
        if let Some(context) = &self.context
            && record.context != *context
        {
            return false;
        }
        if !self.outline_path_prefix.is_empty()
            && !record.outline_path.starts_with(&self.outline_path_prefix)
        {
            return false;
        }
        self.summary_equals.iter().all(|predicate| {
            record
                .summary
                .get(&predicate.key)
                .is_some_and(|value| value == &predicate.value)
        }) && self.summary_contains.iter().all(|predicate| {
            record
                .summary
                .get(&predicate.key)
                .is_some_and(|value| value.contains_text(&predicate.needle))
        })
    }
}

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

impl From<&str> for OrgElementsIndexKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for OrgElementsIndexKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// High-level category for an Org elements index record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgElementsIndexCategory {
    Document,
    Section,
    Element,
    Object,
    Keyword,
    Property,
    TargetDefinition,
    FootnoteEntry,
}

impl OrgElementsIndexCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Section => "section",
            Self::Element => "element",
            Self::Object => "object",
            Self::Keyword => "keyword",
            Self::Property => "property",
            Self::TargetDefinition => "target-definition",
            Self::FootnoteEntry => "footnote-entry",
        }
    }

    pub fn from_label(value: &str) -> Option<Self> {
        match value {
            "document" => Some(Self::Document),
            "section" => Some(Self::Section),
            "element" => Some(Self::Element),
            "object" => Some(Self::Object),
            "keyword" => Some(Self::Keyword),
            "property" => Some(Self::Property),
            "target-definition" => Some(Self::TargetDefinition),
            "footnote-entry" => Some(Self::FootnoteEntry),
            _ => None,
        }
    }
}

/// Compact per-kind summary fields for a flat Org elements index record.
pub type OrgElementsIndexSummary = BTreeMap<String, OrgElementsIndexSummaryValue>;

/// JSON-compatible scalar or small-list value used by `OrgElementsIndexSummary`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgElementsIndexSummaryValue {
    Null,
    Bool(bool),
    Integer(i64),
    Text(String),
    StringList(Vec<String>),
}

impl OrgElementsIndexSummaryValue {
    fn contains_text(&self, needle: &str) -> bool {
        match self {
            Self::Text(value) => value.contains(needle),
            Self::StringList(values) => values.iter().any(|value| value.contains(needle)),
            Self::Null | Self::Bool(_) | Self::Integer(_) => false,
        }
    }
}

/// Exact-match predicate over one compact Org elements index summary field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsIndexSummaryPredicate {
    pub key: String,
    pub value: OrgElementsIndexSummaryValue,
}

/// Text substring predicate over one compact Org elements index summary field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsIndexSummaryTextPredicate {
    pub key: String,
    pub needle: String,
}

impl From<bool> for OrgElementsIndexSummaryValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<usize> for OrgElementsIndexSummaryValue {
    fn from(value: usize) -> Self {
        Self::Integer(value as i64)
    }
}

impl From<u8> for OrgElementsIndexSummaryValue {
    fn from(value: u8) -> Self {
        Self::Integer(i64::from(value))
    }
}

impl From<&str> for OrgElementsIndexSummaryValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for OrgElementsIndexSummaryValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&String> for OrgElementsIndexSummaryValue {
    fn from(value: &String) -> Self {
        Self::Text(value.clone())
    }
}

impl From<Vec<String>> for OrgElementsIndexSummaryValue {
    fn from(value: Vec<String>) -> Self {
        Self::StringList(value)
    }
}

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

/// One executable Python directive from `#+PYTHON:` or `#+PYTHON_FILE:`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonDirective<A = ()> {
    pub ann: A,
    pub kind: PythonDirectiveKind,
    pub value: String,
    pub raw: String,
}

/// Supported Python directive sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PythonDirectiveKind {
    /// Inline Python code from `#+PYTHON:`.
    Inline,
    /// Python script path from `#+PYTHON_FILE:`.
    File,
}

/// Python program selected by an explicit host call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PythonExecutionProgram {
    Inline(String),
    File(String),
}

/// Generic host process selected by an explicit caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsHostExecutionOptions {
    pub command: String,
    pub args: Vec<String>,
}

impl OrgElementsHostExecutionOptions {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}

/// Options for running Python with a JSON Org elements payload on stdin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonExecutionOptions {
    pub interpreter: String,
    pub isolated: bool,
    pub program: PythonExecutionProgram,
}

impl PythonExecutionOptions {
    pub fn inline(code: impl Into<String>) -> Self {
        Self {
            interpreter: "python3".to_string(),
            isolated: true,
            program: PythonExecutionProgram::Inline(code.into()),
        }
    }

    pub fn file(path: impl Into<String>) -> Self {
        Self {
            interpreter: "python3".to_string(),
            isolated: true,
            program: PythonExecutionProgram::File(path.into()),
        }
    }

    pub fn with_interpreter(mut self, interpreter: impl Into<String>) -> Self {
        self.interpreter = interpreter.into();
        self
    }

    pub fn without_isolated(mut self) -> Self {
        self.isolated = false;
        self
    }

    pub fn to_host_options(&self) -> OrgElementsHostExecutionOptions {
        let mut options = OrgElementsHostExecutionOptions::new(self.interpreter.clone());
        if self.isolated {
            options.args.push("-I".to_string());
        }
        match &self.program {
            PythonExecutionProgram::Inline(code) => {
                options.args.push("-c".to_string());
                options.args.push(code.clone());
            }
            PythonExecutionProgram::File(path) => {
                options.args.push(path.clone());
            }
        }
        options
    }
}

/// Exit status from a host execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsHostExecutionStatus {
    pub success: bool,
    pub code: Option<i32>,
}

/// Captured output from a host execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementsHostExecutionOutput {
    pub status: OrgElementsHostExecutionStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Host process error while starting or communicating with a tool.
#[derive(Debug)]
pub enum OrgElementsHostExecutionError {
    Spawn(std::io::Error),
    Stdin(std::io::Error),
    Wait(std::io::Error),
}

impl fmt::Display for OrgElementsHostExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(error) => write!(f, "failed to start Org elements host: {error}"),
            Self::Stdin(error) => write!(f, "failed to write Org elements to host: {error}"),
            Self::Wait(error) => write!(f, "failed to wait for Org elements host: {error}"),
        }
    }
}

impl std::error::Error for OrgElementsHostExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(error) | Self::Stdin(error) | Self::Wait(error) => Some(error),
        }
    }
}
