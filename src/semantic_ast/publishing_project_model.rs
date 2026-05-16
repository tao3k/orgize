//! Non-executing publishing project graph DTOs.

use super::SectionIndexSource;

/// Explicit publishing project input supplied by a caller/config snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingProjectConfig {
    pub name: String,
    pub source_root: String,
    pub publishing_directory: String,
    pub recursive: bool,
    pub sitemap: bool,
}

impl PublishingProjectConfig {
    /// Creates a project config with conservative defaults.
    pub fn new(
        name: impl Into<String>,
        source_root: impl Into<String>,
        publishing_directory: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            source_root: source_root.into(),
            publishing_directory: publishing_directory.into(),
            recursive: true,
            sitemap: false,
        }
    }

    /// Enables or disables recursive source selection.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Enables or disables sitemap planning.
    pub fn sitemap(mut self, sitemap: bool) -> Self {
        self.sitemap = sitemap;
        self
    }
}

/// Planned publishing graph. It never writes files or runs exporters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingProjectPlan {
    pub config: PublishingProjectConfig,
    pub documents: Vec<PublishingProjectDocument>,
    pub dependencies: Vec<PublishingDependency>,
    pub sitemap: Option<PublishingSitemapPlan>,
    pub warnings: Vec<PublishingProjectWarning>,
}

/// One source document in a publishing project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingProjectDocument {
    pub source_file: String,
    pub output_file: String,
    pub title: Option<String>,
    pub source: Option<SectionIndexSource>,
}

/// One publish-time dependency edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingDependency {
    pub source_file: String,
    pub kind: PublishingDependencyKind,
    pub target: String,
}

/// Stable publishing dependency category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishingDependencyKind {
    ProjectRoot,
    Include,
    SetupFile,
    Bibliography,
    Macro,
}

impl PublishingDependencyKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectRoot => "projectRoot",
            Self::Include => "include",
            Self::SetupFile => "setupFile",
            Self::Bibliography => "bibliography",
            Self::Macro => "macro",
        }
    }
}

/// Sitemap planning metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingSitemapPlan {
    pub output_file: String,
    pub entries: Vec<PublishingSitemapEntry>,
}

/// One sitemap entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingSitemapEntry {
    pub source_file: String,
    pub output_file: String,
    pub title: String,
}

/// Non-fatal publishing project warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishingProjectWarning {
    pub kind: PublishingProjectWarningKind,
    pub message: String,
}

/// Stable publishing warning category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishingProjectWarningKind {
    EmptyProject,
}

impl PublishingProjectWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyProject => "emptyProject",
        }
    }
}
