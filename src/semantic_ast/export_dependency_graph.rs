//! Non-executing include/macro/export dependency graph projection.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::{
    CitationExportPlan, Document, ExportDependencyDiagnostic, ExportDependencyDiagnosticKind,
    ExportDependencyEdge, ExportDependencyEdgeKind, ExportDependencyGraph,
    ExportDependencyGraphOptions, ExportDependencyNode, ExportDependencyNodeKind,
    IncludeExpansionOptions, MacroExpansionStatus, ParsedAnnotation,
};

impl ExportDependencyGraph {
    /// Builds a graph from parsed documents and their semantic side tables.
    pub fn build<'a>(
        documents: impl IntoIterator<Item = (&'a str, &'a Document<ParsedAnnotation>)>,
        options: &ExportDependencyGraphOptions,
    ) -> Self {
        let documents = documents.into_iter().collect::<Vec<_>>();
        let document_paths = documents
            .iter()
            .map(|(source_file, _)| normalize_local_path(source_file))
            .collect::<BTreeSet<_>>();
        let global_macros = documents
            .iter()
            .flat_map(|(_, document)| {
                document
                    .macro_definitions
                    .iter()
                    .map(|definition| definition.name.clone())
            })
            .collect::<BTreeSet<_>>();

        let mut builder = ExportDependencyGraphBuilder::default();
        for source_file in &document_paths {
            builder.add_node(
                doc_id(source_file),
                ExportDependencyNodeKind::Document,
                source_file.clone(),
            );
        }

        for (source_file, document) in documents {
            collect_document(
                normalize_local_path(source_file).as_str(),
                document,
                options,
                &document_paths,
                &global_macros,
                &mut builder,
            );
        }

        let mut graph = builder.finish();
        graph.diagnostics.extend(cycle_diagnostics(&graph));
        graph
    }
}

/// Builds a graph from parsed documents and their semantic side tables.
pub fn export_dependency_graph<'a>(
    documents: impl IntoIterator<Item = (&'a str, &'a Document<ParsedAnnotation>)>,
    options: &ExportDependencyGraphOptions,
) -> ExportDependencyGraph {
    ExportDependencyGraph::build(documents, options)
}

impl Document<ParsedAnnotation> {
    /// Builds a dependency graph for one parsed document.
    pub fn export_dependency_graph(
        &self,
        source_file: impl Into<String>,
        options: &ExportDependencyGraphOptions,
    ) -> ExportDependencyGraph {
        let source_file = source_file.into();
        ExportDependencyGraph::build([(source_file.as_str(), self)], options)
    }
}

fn collect_document(
    source_file: &str,
    document: &Document<ParsedAnnotation>,
    options: &ExportDependencyGraphOptions,
    document_paths: &BTreeSet<String>,
    global_macros: &BTreeSet<String>,
    builder: &mut ExportDependencyGraphBuilder,
) {
    let source_id = doc_id(source_file);
    let base_dir = dependency_base_dir(source_file, options);
    let include_options = IncludeExpansionOptions {
        base_dir: base_dir.clone(),
    };
    for include in document
        .include_expansion_plan(&include_options)
        .entries
        .into_iter()
    {
        let target = include.resolved_path.unwrap_or(include.directive.path);
        builder.add_file_dependency(
            source_id.as_str(),
            &target,
            FileDependencyKind::Include,
            options.validate_paths,
            document_paths,
        );
    }

    let settings = document.publishing_settings();
    for setup in settings.setup_files {
        let target = resolve_dependency_path(setup.value.as_str(), base_dir.as_deref());
        builder.add_file_dependency(
            source_id.as_str(),
            target.as_str(),
            FileDependencyKind::SetupFile,
            options.validate_paths,
            document_paths,
        );
    }

    let citations: CitationExportPlan<ParsedAnnotation> = document.citation_export_plan();
    for target in citations
        .bibliographies
        .into_iter()
        .flat_map(|bibliography| bibliography.files)
    {
        let target = resolve_dependency_path(target.as_str(), base_dir.as_deref());
        builder.add_file_dependency(
            source_id.as_str(),
            target.as_str(),
            FileDependencyKind::Bibliography,
            options.validate_paths,
            document_paths,
        );
    }

    for definition in &document.macro_definitions {
        let target_id = macro_id(definition.name.as_str());
        builder.add_node(
            target_id.clone(),
            ExportDependencyNodeKind::Macro,
            definition.name.clone(),
        );
        builder.add_edge(
            source_id.clone(),
            target_id,
            ExportDependencyEdgeKind::DefinesMacro,
        );
    }

    for expansion in document.macro_expansions() {
        let target_id = macro_id(expansion.name.as_str());
        builder.add_node(
            target_id.clone(),
            ExportDependencyNodeKind::Macro,
            expansion.name.clone(),
        );
        builder.add_edge(
            source_id.clone(),
            target_id,
            ExportDependencyEdgeKind::UsesMacro,
        );
        if expansion.status == MacroExpansionStatus::MissingDefinition
            && !global_macros.contains(&expansion.name)
        {
            builder.add_diagnostic(
                ExportDependencyDiagnosticKind::MissingMacroDefinition,
                expansion.name.clone(),
                format!(
                    "macro `{}` is used by `{source_file}` but no parsed document defines it",
                    expansion.name
                ),
            );
        }
    }

    if let Some(output) = publishing_output_path(source_file, document, options) {
        let output_id = output_id(output.as_str());
        builder.add_node(
            output_id.clone(),
            ExportDependencyNodeKind::PublishingOutput,
            output,
        );
        builder.add_edge(source_id, output_id, ExportDependencyEdgeKind::PublishesTo);
    }
}

#[derive(Clone, Copy)]
enum FileDependencyKind {
    Include,
    SetupFile,
    Bibliography,
}

impl FileDependencyKind {
    const fn node_kind(self) -> ExportDependencyNodeKind {
        match self {
            Self::Include => ExportDependencyNodeKind::Include,
            Self::SetupFile => ExportDependencyNodeKind::SetupFile,
            Self::Bibliography => ExportDependencyNodeKind::Bibliography,
        }
    }

    const fn edge_kind(self) -> ExportDependencyEdgeKind {
        match self {
            Self::Include => ExportDependencyEdgeKind::Includes,
            Self::SetupFile => ExportDependencyEdgeKind::UsesSetupFile,
            Self::Bibliography => ExportDependencyEdgeKind::UsesBibliography,
        }
    }

    const fn resource_prefix(self) -> &'static str {
        match self {
            Self::Include => "include",
            Self::SetupFile => "setup",
            Self::Bibliography => "bibliography",
        }
    }
}

#[derive(Default)]
struct ExportDependencyGraphBuilder {
    nodes: BTreeMap<String, ExportDependencyNode>,
    edges: BTreeSet<ExportDependencyEdge>,
    diagnostics: Vec<ExportDependencyDiagnostic>,
}

impl ExportDependencyGraphBuilder {
    fn add_node(&mut self, id: String, kind: ExportDependencyNodeKind, label: String) {
        self.nodes
            .entry(id.clone())
            .or_insert(ExportDependencyNode { id, kind, label });
    }

    fn add_edge(&mut self, source: String, target: String, kind: ExportDependencyEdgeKind) {
        self.edges.insert(ExportDependencyEdge {
            source,
            target,
            kind,
        });
    }

    fn add_file_dependency(
        &mut self,
        source_id: &str,
        target: &str,
        kind: FileDependencyKind,
        validate_paths: bool,
        document_paths: &BTreeSet<String>,
    ) {
        let target = normalize_local_path(trim_wrapping_quotes(target));
        let target_id = if document_paths.contains(&target) {
            doc_id(target.as_str())
        } else {
            let target_id = resource_id(kind.resource_prefix(), target.as_str());
            self.add_node(target_id.clone(), kind.node_kind(), target.clone());
            target_id
        };
        self.add_edge(source_id.to_string(), target_id, kind.edge_kind());

        if validate_paths
            && !document_paths.contains(&target)
            && is_local_filesystem_path(&target)
            && !Path::new(&target).exists()
        {
            self.add_diagnostic(
                ExportDependencyDiagnosticKind::MissingPath,
                target.clone(),
                format!(
                    "{} dependency `{target}` does not exist",
                    kind.node_kind().as_str()
                ),
            );
        }
    }

    fn add_diagnostic(
        &mut self,
        kind: ExportDependencyDiagnosticKind,
        subject: String,
        message: String,
    ) {
        self.diagnostics.push(ExportDependencyDiagnostic {
            kind,
            subject,
            message,
        });
    }

    fn finish(self) -> ExportDependencyGraph {
        ExportDependencyGraph {
            nodes: self.nodes.into_values().collect(),
            edges: self.edges.into_iter().collect(),
            diagnostics: self.diagnostics,
        }
    }
}

fn cycle_diagnostics(graph: &ExportDependencyGraph) -> Vec<ExportDependencyDiagnostic> {
    let document_ids = graph
        .nodes
        .iter()
        .filter(|node| node.kind == ExportDependencyNodeKind::Document)
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in &graph.edges {
        if matches!(
            edge.kind,
            ExportDependencyEdgeKind::Includes | ExportDependencyEdgeKind::UsesSetupFile
        ) && document_ids.contains(&edge.source)
            && document_ids.contains(&edge.target)
        {
            adjacency
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }
    }

    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for document_id in &document_ids {
        let mut path = Vec::new();
        collect_cycles_from(
            document_id,
            document_id,
            &adjacency,
            &mut path,
            &mut seen,
            &mut diagnostics,
        );
    }
    diagnostics
}

fn collect_cycles_from(
    start: &str,
    current: &str,
    adjacency: &BTreeMap<String, Vec<String>>,
    path: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
    diagnostics: &mut Vec<ExportDependencyDiagnostic>,
) {
    path.push(current.to_string());
    if let Some(next_ids) = adjacency.get(current) {
        for next_id in next_ids {
            if next_id == start && path.len() > 1 {
                let mut cycle = path.clone();
                cycle.push(start.to_string());
                let key = canonical_cycle_key(&cycle);
                if seen.insert(key) {
                    let subject = cycle
                        .iter()
                        .map(|id| id.trim_start_matches("doc:"))
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    diagnostics.push(ExportDependencyDiagnostic {
                        kind: ExportDependencyDiagnosticKind::DependencyCycle,
                        subject: subject.clone(),
                        message: format!("export dependency cycle detected: {subject}"),
                    });
                }
            } else if !path.contains(next_id) {
                collect_cycles_from(start, next_id, adjacency, path, seen, diagnostics);
            }
        }
    }
    let _ = path.pop();
}

fn canonical_cycle_key(cycle: &[String]) -> String {
    let nodes = &cycle[..cycle.len().saturating_sub(1)];
    if nodes.is_empty() {
        return String::new();
    }
    let min_index = nodes
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0);
    (0..nodes.len())
        .map(|offset| nodes[(min_index + offset) % nodes.len()].as_str())
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn publishing_output_path(
    source_file: &str,
    document: &Document<ParsedAnnotation>,
    options: &ExportDependencyGraphOptions,
) -> Option<String> {
    let settings = document.publishing_settings();
    if let Some(export_file_name) = settings.export_file_name {
        return Some(export_file_name.value);
    }
    options
        .publishing_directory
        .as_deref()
        .map(|directory| join_path(directory, html_output_path(source_file).as_str()))
}

fn dependency_base_dir(
    source_file: &str,
    options: &ExportDependencyGraphOptions,
) -> Option<String> {
    options
        .base_dir
        .clone()
        .or_else(|| parent_directory(source_file))
}

fn resolve_dependency_path(path: &str, base_dir: Option<&str>) -> String {
    let path = trim_wrapping_quotes(path);
    if is_absolute_or_special_path(path) {
        normalize_local_path(path)
    } else if let Some(base_dir) = base_dir.filter(|base_dir| !base_dir.is_empty()) {
        normalize_local_path(format!("{base_dir}/{path}").as_str())
    } else {
        normalize_local_path(path)
    }
}

fn trim_wrapping_quotes(value: &str) -> &str {
    value
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or_else(|| {
            value
                .trim()
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .unwrap_or(value.trim())
        })
}

fn parent_directory(path: &str) -> Option<String> {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .filter(|parent| !parent.is_empty())
}

fn html_output_path(source_file: &str) -> String {
    source_file
        .strip_suffix(".org")
        .map(|stem| format!("{stem}.html"))
        .unwrap_or_else(|| format!("{source_file}.html"))
}

fn join_path(prefix: &str, suffix: &str) -> String {
    let prefix = prefix.trim_end_matches('/');
    let suffix = suffix.trim_start_matches('/');
    if prefix.is_empty() {
        suffix.to_string()
    } else {
        format!("{prefix}/{suffix}")
    }
}

fn normalize_local_path(path: &str) -> String {
    if path.contains("://") {
        return path.to_string();
    }
    let path = path.trim();
    let absolute = path.starts_with('/');
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.last().is_some_and(|last| *last != "..") {
                    let _ = parts.pop();
                } else if !absolute {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    let normalized = parts.join("/");
    if absolute {
        format!("/{normalized}")
    } else if normalized.is_empty() {
        ".".to_string()
    } else {
        normalized
    }
}

fn is_absolute_or_special_path(path: &str) -> bool {
    path.starts_with('/') || path.starts_with("~/") || path.contains("://")
}

fn is_local_filesystem_path(path: &str) -> bool {
    !path.starts_with("~/") && !path.contains("://")
}

fn doc_id(source_file: &str) -> String {
    resource_id("doc", source_file)
}

fn macro_id(name: &str) -> String {
    resource_id("macro", name)
}

fn output_id(path: &str) -> String {
    resource_id("output", path)
}

fn resource_id(prefix: &str, value: &str) -> String {
    format!("{prefix}:{value}")
}
