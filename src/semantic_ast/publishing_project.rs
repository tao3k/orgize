//! Non-executing publishing project graph projection.

use super::{
    CitationExportPlan, Document, Keyword, ParsedAnnotation, PublishingDependency,
    PublishingDependencyKind, PublishingProjectConfig, PublishingProjectDocument,
    PublishingProjectPlan, PublishingProjectWarning, PublishingProjectWarningKind,
    PublishingSitemapEntry, PublishingSitemapPlan,
};

impl PublishingProjectPlan {
    /// Builds a publishing graph from explicit project config and parsed docs.
    pub fn build<'a>(
        config: PublishingProjectConfig,
        documents: impl IntoIterator<Item = (&'a str, &'a Document<ParsedAnnotation>)>,
    ) -> Self {
        let mut docs = Vec::new();
        let mut dependencies = Vec::new();
        for (source_file, document) in documents {
            docs.push(publishing_document(source_file, document));
            dependencies.extend(document.export_dependency_edges(source_file));
            dependencies.push(PublishingDependency {
                source_file: source_file.to_string(),
                kind: PublishingDependencyKind::ProjectRoot,
                target: config.source_root.clone(),
            });
        }
        let sitemap = config.sitemap.then(|| PublishingSitemapPlan {
            output_file: join_path(config.publishing_directory.as_str(), "sitemap.org"),
            entries: docs
                .iter()
                .map(|document| PublishingSitemapEntry {
                    source_file: document.source_file.clone(),
                    output_file: document.output_file.clone(),
                    title: document
                        .title
                        .clone()
                        .unwrap_or_else(|| document.source_file.clone()),
                })
                .collect(),
        });
        let warnings = if docs.is_empty() {
            vec![PublishingProjectWarning {
                kind: PublishingProjectWarningKind::EmptyProject,
                message: "publishing project has no parsed documents".to_string(),
            }]
        } else {
            Vec::new()
        };
        Self {
            config,
            documents: docs,
            dependencies,
            sitemap,
            warnings,
        }
    }
}

/// Builds a publishing graph from explicit project config and parsed docs.
pub fn publishing_project_plan<'a>(
    config: PublishingProjectConfig,
    documents: impl IntoIterator<Item = (&'a str, &'a Document<ParsedAnnotation>)>,
) -> PublishingProjectPlan {
    PublishingProjectPlan::build(config, documents)
}

fn publishing_document(
    source_file: &str,
    document: &Document<ParsedAnnotation>,
) -> PublishingProjectDocument {
    let settings = document.publishing_settings();
    let output_file = settings
        .export_file_name
        .as_ref()
        .map(|keyword| keyword.value.clone())
        .unwrap_or_else(|| html_output_path(source_file));
    PublishingProjectDocument {
        source_file: source_file.to_string(),
        output_file,
        title: title_keyword(&document.metadata),
        source: document
            .sections
            .first()
            .map(|section| super::SectionIndexSource::from_annotation(&section.ann)),
    }
}

impl Document<ParsedAnnotation> {
    /// Collects publish-time dependency edges visible in this document.
    pub fn export_dependency_edges(
        &self,
        source_file: impl Into<String>,
    ) -> Vec<PublishingDependency> {
        let source_file = source_file.into();
        let mut dependencies = Vec::new();
        let settings = self.publishing_settings();
        dependencies.extend(
            settings
                .includes
                .into_iter()
                .map(|include| PublishingDependency {
                    source_file: source_file.clone(),
                    kind: PublishingDependencyKind::Include,
                    target: include.path,
                }),
        );
        dependencies.extend(
            settings
                .setup_files
                .into_iter()
                .map(|setup| PublishingDependency {
                    source_file: source_file.clone(),
                    kind: PublishingDependencyKind::SetupFile,
                    target: setup.value,
                }),
        );
        let citations: CitationExportPlan<ParsedAnnotation> = self.citation_export_plan();
        dependencies.extend(
            citations
                .bibliographies
                .into_iter()
                .flat_map(|bibliography| bibliography.files)
                .map(|target| PublishingDependency {
                    source_file: source_file.clone(),
                    kind: PublishingDependencyKind::Bibliography,
                    target,
                }),
        );
        dependencies.extend(self.macro_definitions.iter().map(|macro_definition| {
            PublishingDependency {
                source_file: source_file.clone(),
                kind: PublishingDependencyKind::Macro,
                target: macro_definition.name.clone(),
            }
        }));
        dependencies
    }
}

fn title_keyword(metadata: &[Keyword<ParsedAnnotation>]) -> Option<String> {
    metadata
        .iter()
        .find(|keyword| keyword.key.eq_ignore_ascii_case("TITLE"))
        .map(|keyword| keyword.value.trim().to_string())
        .filter(|title| !title.is_empty())
}

fn html_output_path(source_file: &str) -> String {
    source_file
        .strip_suffix(".org")
        .map(|stem| format!("{stem}.html"))
        .unwrap_or_else(|| format!("{source_file}.html"))
}

fn join_path(prefix: &str, suffix: &str) -> String {
    let prefix = prefix.trim_end_matches('/');
    if prefix.is_empty() {
        suffix.to_string()
    } else {
        format!("{prefix}/{suffix}")
    }
}
