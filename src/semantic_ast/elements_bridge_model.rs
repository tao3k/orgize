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
    pub id: OrgElementId,
    pub parent_id: Option<OrgElementId>,
    pub child_ids: Vec<OrgElementId>,
    pub ann: A,
    pub ordinal: usize,
    pub category: OrgElementsIndexCategory,
    pub kind: OrgElementsIndexKind,
    pub affiliated: OrgElementsAffiliatedProperties,
    pub outline_path: Vec<String>,
    pub context: String,
    pub properties: OrgElementProperties,
    pub property_provenance: OrgElementPropertyProvenanceMap,
    pub summary: OrgElementsIndexSummary,
}

/// Stable identifier for a record in the Org elements graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrgElementId(usize);

impl OrgElementId {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn as_usize(self) -> usize {
        self.0
    }
}

/// Scope rooted at one Org element graph record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrgElementScope {
    pub root_id: OrgElementId,
}

/// Org-mode-style property value used by element queries.
pub type OrgElementValue = OrgElementsIndexSummaryValue;

/// Org-mode-style property map used by element queries.
pub type OrgElementProperties = BTreeMap<String, OrgElementValue>;

/// Provenance map for properties projected onto one Org elements index record.
pub type OrgElementPropertyProvenanceMap = BTreeMap<String, OrgElementPropertyProvenance>;

/// Parser-owned provenance for one projected Org element property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgElementPropertyProvenance {
    Summary,
    Standard,
    Local,
    Effective,
    Inherited,
}

impl OrgElementPropertyProvenance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Standard => "standard",
            Self::Local => "local",
            Self::Effective => "effective",
            Self::Inherited => "inherited",
        }
    }
}

/// Parent/child graph over the same records returned by the flat index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgElementGraph<A = ()> {
    pub records: Vec<OrgElementsIndexRecord<A>>,
    pub by_id: BTreeMap<OrgElementId, usize>,
    pub root_id: OrgElementId,
}

/// Org-mode-style properties derived from affiliated keywords on an element.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgElementsAffiliatedProperties {
    pub name: Option<String>,
}

/// Stable node kind label in the Org elements flat index.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrgElementsIndexKind(String);

pub const ORGIZE_ORG_ELEMENT_EXTENSION_NAMESPACE: &str = "orgize";

/// Namespace ownership for an Org elements index kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgElementKindNamespace {
    Upstream,
    OrgizeExtension,
}

impl OrgElementKindNamespace {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Upstream => "upstream",
            Self::OrgizeExtension => "orgize-extension",
        }
    }
}

impl OrgElementsIndexKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn namespace(&self) -> OrgElementKindNamespace {
        match self.as_str() {
            "plain-text" => OrgElementKindNamespace::OrgizeExtension,
            _ => OrgElementKindNamespace::Upstream,
        }
    }

    pub fn extension_namespace(&self) -> Option<&'static str> {
        (self.namespace() == OrgElementKindNamespace::OrgizeExtension)
            .then_some(ORGIZE_ORG_ELEMENT_EXTENSION_NAMESPACE)
    }
}

impl<A> OrgElementGraph<A> {
    pub fn new(records: Vec<OrgElementsIndexRecord<A>>) -> Self {
        let root_id = records
            .first()
            .map(|record| record.id)
            .unwrap_or_else(|| OrgElementId::new(0));
        let by_id = records
            .iter()
            .enumerate()
            .map(|(index, record)| (record.id, index))
            .collect();
        Self {
            records,
            by_id,
            root_id,
        }
    }

    pub fn record(&self, id: OrgElementId) -> Option<&OrgElementsIndexRecord<A>> {
        self.by_id
            .get(&id)
            .and_then(|index| self.records.get(*index))
    }

    pub fn parent(&self, id: OrgElementId) -> Option<&OrgElementsIndexRecord<A>> {
        self.record(id)
            .and_then(|record| record.parent_id)
            .and_then(|parent_id| self.record(parent_id))
    }

    pub fn children(&self, id: OrgElementId) -> Vec<&OrgElementsIndexRecord<A>> {
        self.record(id)
            .map(|record| {
                record
                    .child_ids
                    .iter()
                    .filter_map(|child_id| self.record(*child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn descendants(&self, id: OrgElementId) -> Vec<&OrgElementsIndexRecord<A>> {
        let mut descendants = Vec::new();
        self.collect_descendants(id, &mut descendants);
        descendants
    }

    pub fn ancestors(&self, id: OrgElementId) -> Vec<&OrgElementsIndexRecord<A>> {
        let mut ancestors = Vec::new();
        let mut cursor = self.record(id).and_then(|record| record.parent_id);
        while let Some(parent_id) = cursor {
            let Some(parent) = self.record(parent_id) else {
                break;
            };
            ancestors.push(parent);
            cursor = parent.parent_id;
        }
        ancestors
    }

    pub fn lineage(&self, id: OrgElementId) -> Vec<&OrgElementsIndexRecord<A>> {
        let mut lineage = self.ancestors(id);
        lineage.reverse();
        if let Some(record) = self.record(id) {
            lineage.push(record);
        }
        lineage
    }

    pub fn subtree(&self, id: OrgElementId) -> OrgElementScope {
        OrgElementScope { root_id: id }
    }

    fn collect_descendants<'a>(
        &'a self,
        id: OrgElementId,
        descendants: &mut Vec<&'a OrgElementsIndexRecord<A>>,
    ) {
        for child in self.children(id) {
            descendants.push(child);
            self.collect_descendants(child.id, descendants);
        }
    }
}

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
