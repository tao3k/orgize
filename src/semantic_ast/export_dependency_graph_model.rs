//! Non-executing export dependency graph DTOs.

/// Options for building an export dependency graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportDependencyGraphOptions {
    pub base_dir: Option<String>,
    pub publishing_directory: Option<String>,
    pub validate_paths: bool,
}

impl ExportDependencyGraphOptions {
    /// Creates graph options with conservative defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves relative dependency paths against a fixed base directory.
    pub fn with_base_dir(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
            ..Self::default()
        }
    }

    /// Sets the publishing output directory used for default output edges.
    pub fn publishing_directory(mut self, publishing_directory: impl Into<String>) -> Self {
        self.publishing_directory = Some(publishing_directory.into());
        self
    }

    /// Enables or disables local path existence checks.
    pub fn validate_paths(mut self, validate_paths: bool) -> Self {
        self.validate_paths = validate_paths;
        self
    }
}

/// Combined include/macro/export dependency graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportDependencyGraph {
    pub nodes: Vec<ExportDependencyNode>,
    pub edges: Vec<ExportDependencyEdge>,
    pub diagnostics: Vec<ExportDependencyDiagnostic>,
}

/// One graph node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportDependencyNode {
    pub id: String,
    pub kind: ExportDependencyNodeKind,
    pub label: String,
}

/// Stable graph node category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExportDependencyNodeKind {
    Document,
    Include,
    SetupFile,
    Bibliography,
    Macro,
    PublishingOutput,
}

impl ExportDependencyNodeKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Include => "include",
            Self::SetupFile => "setupFile",
            Self::Bibliography => "bibliography",
            Self::Macro => "macro",
            Self::PublishingOutput => "publishingOutput",
        }
    }
}

/// One graph edge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExportDependencyEdge {
    pub source: String,
    pub target: String,
    pub kind: ExportDependencyEdgeKind,
}

/// Stable graph edge category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExportDependencyEdgeKind {
    Includes,
    UsesSetupFile,
    UsesBibliography,
    DefinesMacro,
    UsesMacro,
    PublishesTo,
}

impl ExportDependencyEdgeKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Includes => "includes",
            Self::UsesSetupFile => "usesSetupFile",
            Self::UsesBibliography => "usesBibliography",
            Self::DefinesMacro => "definesMacro",
            Self::UsesMacro => "usesMacro",
            Self::PublishesTo => "publishesTo",
        }
    }
}

/// Non-fatal graph diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportDependencyDiagnostic {
    pub kind: ExportDependencyDiagnosticKind,
    pub subject: String,
    pub message: String,
}

/// Stable graph diagnostic category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExportDependencyDiagnosticKind {
    MissingPath,
    DependencyCycle,
    MissingMacroDefinition,
}

impl ExportDependencyDiagnosticKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingPath => "missingPath",
            Self::DependencyCycle => "dependencyCycle",
            Self::MissingMacroDefinition => "missingMacroDefinition",
        }
    }
}
