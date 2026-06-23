//! Contract model for `CONTRACT_ORG` validation over Org element index records.

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use rowan::TextRange;

use super::{
    OrgElementId, OrgElementQueryPredicate, OrgElementsIndexCategory, OrgElementsIndexKind,
    OrgElementsIndexQuery, SourceBlockSource,
};

/// File-level property naming a contract registry reference.
pub const CONTRACT_ORG_PROPERTY: &str = "CONTRACT_ORG";

/// Contract identity property on top-level contract definitions.
pub const CONTRACT_ID_PROPERTY: &str = "CONTRACT_ID";

/// Optional list of contract aliases.
pub const CONTRACT_ALIAS_PROPERTY: &str = "CONTRACT_ALIAS";

/// Contract scope property (for example `document` or `subtree`).
pub const CONTRACT_SCOPE_PROPERTY: &str = "CONTRACT_SCOPE";

/// Contract type property.
pub const CONTRACT_KIND_PROPERTY: &str = "CONTRACT_KIND";

/// Assertion identifier property inside assertion headings.
pub const ASSERT_ID_PROPERTY: &str = "ASSERT_ID";

/// Assertion severity property inside assertion headings.
pub const ASSERT_SEVERITY_PROPERTY: &str = "SEVERITY";

/// Supported contract kind for this feature.
pub const CONTRACT_KIND_ORG_ELEMENTS: &str = "org-elements";

/// Typed contract kind for `CONTRACT_ORG` documents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgContractKind {
    /// Assertions executed over `Document::org_elements_index()`.
    OrgElementsAssertions,
}

impl OrgContractKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OrgElementsAssertions => CONTRACT_KIND_ORG_ELEMENTS,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            CONTRACT_KIND_ORG_ELEMENTS => Some(Self::OrgElementsAssertions),
            _ => None,
        }
    }
}

/// Registry of parsed `CONTRACT_ORG` contracts available during linting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgContractRegistry {
    pub contracts: Vec<OrgContract>,
}

impl OrgContractRegistry {
    pub fn new(contracts: impl IntoIterator<Item = OrgContract>) -> Self {
        Self {
            contracts: contracts.into_iter().collect(),
        }
    }

    /// Resolves a contract reference from the loaded registry.
    pub fn resolve(&self, reference: &OrgContractReference) -> Option<&OrgContract> {
        self.contracts
            .iter()
            .find(|contract| reference_matches_contract(contract, reference))
    }
}

/// Parsed contract reference from a `CONTRACT_ORG` value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgContractReference {
    pub raw: String,
    pub path: Option<String>,
    pub contract_id: Option<String>,
}

impl OrgContractReference {
    pub fn with_source_relative_path(&self, source_path: Option<&Path>) -> Self {
        let Some(path) = self.path.as_deref() else {
            return self.clone();
        };
        let path = Path::new(path);
        if path.is_absolute() {
            return self.clone();
        }
        let Some(source_parent) = source_path.and_then(Path::parent) else {
            return self.clone();
        };

        let resolved = normalize_lexical_path(&source_parent.join(path));
        if self.path.as_deref() == Some(resolved.as_str()) {
            return self.clone();
        }

        Self {
            raw: self.raw.clone(),
            path: Some(resolved),
            contract_id: self.contract_id.clone(),
        }
    }
}

/// Host-owned assertion scope for applying a contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrgContractScope {
    /// Scope resolves to the full document.
    Document,
    /// Scope resolves to a subtree and applies `outline_path_prefix` implicitly.
    #[default]
    Subtree,
}

impl OrgContractScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Subtree => "subtree",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "document" => Some(Self::Document),
            "subtree" => Some(Self::Subtree),
            _ => None,
        }
    }
}

/// One loaded contract definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgContract {
    pub id: String,
    pub aliases: Vec<String>,
    pub scope: OrgContractScope,
    pub kind: OrgContractKind,
    pub assertions: Vec<OrgContractAssertion>,
}

/// One assertion inside an org contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgContractAssertion {
    pub id: String,
    pub severity: OrgContractSeverity,
    pub bindings: Vec<OrgContractBinding>,
    pub query: OrgContractQuery,
    pub expectation: OrgContractExpectation,
    pub message: Option<String>,
    pub fix: Option<String>,
    pub query_source: Option<SourceBlockSource>,
    pub expect_source: Option<SourceBlockSource>,
}

/// Named query binding declared by an `org-contract` block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgContractBinding {
    pub name: String,
    pub query: OrgContractQuery,
}

/// Evaluation facts for one resolved `CONTRACT_ORG` contract in one scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgContractEvaluation {
    pub contract_id: String,
    pub scope: OrgContractEvaluationScope,
    pub assertions: Vec<OrgContractAssertionEvaluation>,
}

/// Source-backed scope where a contract was evaluated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgContractEvaluationScope {
    Document {
        range: TextRange,
    },
    Section {
        title: String,
        outline_path: Vec<String>,
        range: TextRange,
    },
}

impl OrgContractEvaluationScope {
    pub fn document() -> Self {
        Self::Document {
            range: TextRange::new(0.into(), 0.into()),
        }
    }

    pub fn section(title: impl Into<String>, outline_path: Vec<String>, range: TextRange) -> Self {
        Self::Section {
            title: title.into(),
            outline_path,
            range,
        }
    }

    pub const fn kind_as_str(&self) -> &'static str {
        match self {
            Self::Document { .. } => "document",
            Self::Section { .. } => "section",
        }
    }

    pub fn title(&self) -> Option<&str> {
        match self {
            Self::Document { .. } => None,
            Self::Section { title, .. } => Some(title.as_str()),
        }
    }

    pub fn outline_path(&self) -> &[String] {
        match self {
            Self::Document { .. } => &[],
            Self::Section { outline_path, .. } => outline_path,
        }
    }

    pub const fn range(&self) -> TextRange {
        match self {
            Self::Document { range } | Self::Section { range, .. } => *range,
        }
    }
}

/// Evaluation facts for one assertion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgContractAssertionEvaluation {
    pub assertion_id: String,
    pub severity: OrgContractSeverity,
    pub expectation: OrgContractExpectation,
    pub actual_count: usize,
    pub status: OrgContractAssertionStatus,
    pub matched_ids: Vec<OrgElementId>,
    pub bindings: BTreeMap<String, Vec<OrgElementId>>,
    pub message_template: Option<String>,
    pub fix_template: Option<String>,
}

/// Result of checking one contract assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgContractAssertionStatus {
    Passed,
    Failed,
}

impl OrgContractAssertionStatus {
    pub const fn is_failed(self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// Assertion severity declared in contract source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrgContractSeverity {
    Error,
    #[default]
    Warning,
}

impl OrgContractSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

/// Parsed query for contract execution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgContractQuery {
    pub category: Option<OrgElementsIndexCategory>,
    pub kind: Option<OrgElementsIndexKind>,
    pub affiliated_name: Option<String>,
    pub context: Option<String>,
    pub outline_path_prefix: Vec<String>,
    pub outline_path_exact_len: Option<usize>,
    pub property_equals: Vec<(String, String)>,
    pub property_contains: Vec<(String, String)>,
    pub summary_equals: Vec<(String, String)>,
    pub summary_contains: Vec<(String, String)>,
    pub predicates: Vec<OrgElementQueryPredicate>,
    pub document_predicates: Vec<OrgContractDocumentPredicate>,
    pub limit: Option<usize>,
    pub use_scope_outline_path: bool,
    pub has_outline_path_prefix: bool,
    pub scope_outline_depth: Option<usize>,
    pub relative_to: Option<OrgContractRelativeScope>,
}

/// Host source facts available to document-level contract predicates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrgContractEvaluationContext {
    pub source_path: Option<PathBuf>,
    /// Effective path scope derived from the native Org DIR property.
    ///
    /// Contract evaluation resolves DIR values as path syntax, including Org
    /// macros, host command substitution with `$(...)`, and environment
    /// variables. The parser still preserves the property value as plain Org
    /// text; expansion belongs to contract evaluation.
    pub dir_scope: Option<PathBuf>,
}

impl OrgContractEvaluationContext {
    pub fn with_source_path(path: impl Into<PathBuf>) -> Self {
        Self {
            source_path: Some(path.into()),
            dir_scope: None,
        }
    }

    pub fn with_dir_scope(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir_scope = Some(dir.into());
        self
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    pub fn dir_scope(&self) -> Option<&Path> {
        self.dir_scope.as_deref()
    }
}

/// Predicate over host-owned document context rather than an Org AST element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgContractDocumentPredicate {
    SourcePathEquals(String),
    SourcePathContains(String),
    SourceFilenameEquals(String),
    SourceFilenamePrefix(String),
    SourceFilenameSuffix(String),
    SourceFilenameStemUppercase(bool),
}

impl OrgContractDocumentPredicate {
    pub fn matches_context(&self, context: &OrgContractEvaluationContext) -> bool {
        match self {
            Self::SourcePathEquals(expected) => context
                .source_path()
                .map(path_to_string)
                .is_some_and(|path| path == *expected),
            Self::SourcePathContains(needle) => context
                .source_path()
                .map(path_to_string)
                .is_some_and(|path| path.contains(needle)),
            Self::SourceFilenameEquals(expected) => source_file_name(context)
                .is_some_and(|filename| filename == *expected),
            Self::SourceFilenamePrefix(prefix) => {
                source_file_name(context).is_some_and(|filename| filename.starts_with(prefix))
            }
            Self::SourceFilenameSuffix(suffix) => {
                source_file_name(context).is_some_and(|filename| filename.ends_with(suffix))
            }
            Self::SourceFilenameStemUppercase(expected) => source_file_stem(context)
                .is_some_and(|stem| filename_stem_is_uppercase(&stem) == *expected),
        }
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn normalize_lexical_path(path: &Path) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let last = normalized.components().next_back();
                if matches!(last, Some(Component::Normal(_))) {
                    normalized.pop();
                } else if !matches!(
                    last,
                    Some(Component::RootDir) | Some(Component::Prefix(_))
                ) {
                    normalized.push("..");
                }
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    if normalized.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path_to_string(&normalized)
    }
}

fn source_file_name(context: &OrgContractEvaluationContext) -> Option<String> {
    context
        .source_path()?
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
}

fn source_file_stem(context: &OrgContractEvaluationContext) -> Option<String> {
    context
        .source_path()?
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
}

fn filename_stem_is_uppercase(stem: &str) -> bool {
    !stem.is_empty() && !stem.chars().any(char::is_lowercase)
}

/// Graph relation used to filter a contract query relative to another query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgContractRelativeScope {
    DescendantOfBinding(String),
    ChildOfBinding(String),
    AtBinding(String),
}

impl OrgContractQuery {
    /// Builds an `OrgElementsIndexQuery` from this contract query.
    pub fn to_index_query(&self) -> OrgElementsIndexQuery {
        let mut query = OrgElementsIndexQuery::new();
        if let Some(category) = self.category {
            query = query.category(category);
        }
        if let Some(kind) = &self.kind {
            query = query.kind(kind.clone());
        }
        if let Some(name) = &self.affiliated_name {
            query = query.affiliated_name(name.clone());
        }
        if let Some(context) = &self.context {
            query = query.context(context.clone());
        }
        if !self.outline_path_prefix.is_empty() {
            query = query.outline_path_prefix(self.outline_path_prefix.clone());
        }
        if let Some(outline_path_exact_len) = self.outline_path_exact_len {
            query = query.outline_path_exact_len(outline_path_exact_len);
        }
        for (key, value) in &self.property_equals {
            query = query.property_eq(key.clone(), value.clone());
        }
        for (key, value) in &self.property_contains {
            query = query.property_contains(key.clone(), value.clone());
        }
        for (key, value) in &self.summary_equals {
            query = query.summary_eq(key.clone(), value.clone());
        }
        for (key, value) in &self.summary_contains {
            query = query.summary_contains(key.clone(), value.clone());
        }
        for predicate in &self.predicates {
            query = query.and_predicate(predicate.clone());
        }
        if let Some(limit) = self.limit {
            query = query.limit(limit);
        }
        query
    }

    pub fn apply_subtree_scope_prefix(mut self, outline_path: Vec<String>) -> Self {
        if self.use_scope_outline_path || !self.has_outline_path_prefix {
            if let Some(depth) = self.scope_outline_depth {
                self.outline_path_exact_len = Some(outline_path.len() + depth);
            }
            self.outline_path_prefix = outline_path;
            self.has_outline_path_prefix = true;
        }
        self
    }
}

/// One assertion expectation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrgContractExpectation {
    Exists,
    NotExists,
    Count(OrgContractCompareOp, usize),
}

impl OrgContractExpectation {
    pub fn expected_summary(&self) -> String {
        match self {
            Self::Exists => "exists".to_string(),
            Self::NotExists => "not exists".to_string(),
            Self::Count(op, count) => format!("count {} {}", op.as_str(), count),
        }
    }

    pub fn check(&self, actual: usize) -> bool {
        match self {
            Self::Exists => actual > 0,
            Self::NotExists => actual == 0,
            Self::Count(op, expected) => op.matches(actual, *expected),
        }
    }
}

/// Comparison operator for a `count` expectation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrgContractCompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl OrgContractCompareOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }

    fn matches(self, left: usize, right: usize) -> bool {
        match self {
            Self::Eq => left == right,
            Self::Ne => left != right,
            Self::Lt => left < right,
            Self::Le => left <= right,
            Self::Gt => left > right,
            Self::Ge => left >= right,
        }
    }
}

fn reference_matches_contract(contract: &OrgContract, reference: &OrgContractReference) -> bool {
    if let (Some(path), Some(contract_id)) = (&reference.path, &reference.contract_id) {
        return contract_id_matches(contract, contract_id) && contract_path_matches(contract, path);
    }

    if let Some(contract_id) = &reference.contract_id
        && contract_id_matches(contract, contract_id)
    {
        return true;
    }

    if let Some(path) = &reference.path {
        if contract_path_matches(contract, path) {
            return true;
        }
        let prefixed = format!("{path}#{}", contract.id);
        if contract.aliases.iter().any(|alias| alias == &prefixed)
            || contract
                .aliases
                .iter()
                .any(|alias| alias == &format!("file:{path}#{}", contract.id))
        {
            return true;
        }
    }

    if contract.aliases.iter().any(|alias| alias == &reference.raw) {
        return true;
    }

    false
}

fn contract_id_matches(contract: &OrgContract, contract_id: &str) -> bool {
    contract.id == contract_id || contract.aliases.iter().any(|alias| alias == contract_id)
}

fn contract_path_matches(contract: &OrgContract, path: &str) -> bool {
    contract
        .aliases
        .iter()
        .any(|alias| alias == path || alias == &format!("file:{path}"))
}
