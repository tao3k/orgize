//! Org Cite export planning DTOs.

use super::SectionIndexSource;

/// Non-executing citation export plan collected from Org Cite syntax.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationExportPlan<A = ()> {
    pub bibliographies: Vec<CitationBibliography<A>>,
    pub processors: Vec<CitationProcessor<A>>,
    pub print_bibliographies: Vec<PrintBibliography<A>>,
    pub citations: Vec<CitationUsage<A>>,
    pub warnings: Vec<CitationExportWarning>,
}

impl<A> Default for CitationExportPlan<A> {
    fn default() -> Self {
        Self {
            bibliographies: Vec::new(),
            processors: Vec::new(),
            print_bibliographies: Vec::new(),
            citations: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// `#+BIBLIOGRAPHY:` keyword value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationBibliography<A = ()> {
    pub ann: A,
    pub files: Vec<String>,
    pub raw: String,
}

/// `#+CITE_EXPORT:` keyword value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationProcessor<A = ()> {
    pub ann: A,
    pub processor: String,
    pub style: Option<String>,
    pub raw: String,
}

/// `#+PRINT_BIBLIOGRAPHY:` keyword value and parsed plist-like options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintBibliography<A = ()> {
    pub ann: A,
    pub options: Vec<CitationExportOption>,
    pub raw: String,
}

/// One citation object usage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationUsage<A = ()> {
    pub ann: A,
    pub style: String,
    pub variant: String,
    pub keys: Vec<String>,
    pub nocite: bool,
    pub raw: String,
    pub source: Option<SectionIndexSource>,
}

/// Plist-like option from citation export keywords.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationExportOption {
    pub key: String,
    pub value: Option<String>,
    pub raw: String,
}

/// Non-fatal citation export planning warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CitationExportWarning {
    pub kind: CitationExportWarningKind,
    pub message: String,
}

/// Stable warning kind for citation export planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CitationExportWarningKind {
    MissingBibliography,
    PrintBibliographyWithoutProcessor,
}

impl CitationExportWarningKind {
    /// Stable label for DTO and compact consumers.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBibliography => "missingBibliography",
            Self::PrintBibliographyWithoutProcessor => "printBibliographyWithoutProcessor",
        }
    }
}
