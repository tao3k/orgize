//! Cross-document workspace index for Org semantic projections.

use std::collections::BTreeMap;

use super::{
    Document, ParsedAnnotation, SectionIndexRecord, SectionIndexSource, SectionIndexTarget,
    TargetKind,
};
use super::{
    WorkspaceAttachmentKind, WorkspaceAttachmentRef, WorkspaceDocument, WorkspaceDocumentSummary,
    WorkspaceIndex, WorkspaceIssue, WorkspaceIssueKind, WorkspaceLinkRef, WorkspaceResolvedTarget,
    WorkspaceTargetRef,
};

/// Builder for a deterministic workspace-level semantic index.
#[derive(Clone, Debug, Default)]
pub struct WorkspaceIndexBuilder {
    documents: Vec<WorkspaceDocument>,
}

impl WorkspaceIndexBuilder {
    /// Creates an empty workspace index builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one parsed document to the workspace index.
    pub fn add_document(
        &mut self,
        source_file: impl Into<String>,
        document: &Document<ParsedAnnotation>,
    ) -> &mut Self {
        let source_file = source_file.into();
        let sections = document.section_index_records_for_file(source_file.clone());
        let summary = document_summary(&sections, document.source_block_records().len());
        self.documents.push(WorkspaceDocument {
            source_file,
            summary,
            sections,
        });
        self
    }

    /// Finishes the builder and resolves workspace-local internal links.
    pub fn finish(self) -> WorkspaceIndex {
        let documents = self.documents;
        let targets = workspace_targets(&documents);
        let target_map = workspace_target_map(&targets);
        let mut issues = workspace_duplicate_issues(&targets);
        let links = workspace_links(&documents, &target_map, &mut issues);
        let attachments = workspace_attachments(&documents);

        WorkspaceIndex {
            documents,
            targets,
            links,
            attachments,
            issues,
        }
    }
}

impl Document<ParsedAnnotation> {
    /// Builds a single-document workspace index with cross-link diagnostics.
    pub fn workspace_index(&self, source_file: impl Into<String>) -> WorkspaceIndex {
        let mut builder = WorkspaceIndexBuilder::new();
        builder.add_document(source_file, self);
        builder.finish()
    }
}

fn document_summary(
    sections: &[SectionIndexRecord],
    source_block_count: usize,
) -> WorkspaceDocumentSummary {
    WorkspaceDocumentSummary {
        section_count: sections.len(),
        target_count: sections.iter().map(|section| section.targets.len()).sum(),
        link_count: sections.iter().map(|section| section.links.len()).sum(),
        attachment_section_count: sections
            .iter()
            .filter(|section| {
                section.attachment.has_attach_tag || section.attachment.directory.is_some()
            })
            .count(),
        attachment_link_count: sections
            .iter()
            .flat_map(|section| &section.links)
            .filter(|link| link.attachment.is_some())
            .count(),
        source_block_count,
    }
}

fn workspace_targets(documents: &[WorkspaceDocument]) -> Vec<WorkspaceTargetRef> {
    let mut targets = Vec::new();
    for document in documents {
        for section in &document.sections {
            for target in &section.targets {
                targets.push(workspace_target(document, section, target));
            }
        }
    }
    targets
}

fn workspace_target(
    document: &WorkspaceDocument,
    section: &SectionIndexRecord,
    target: &SectionIndexTarget,
) -> WorkspaceTargetRef {
    WorkspaceTargetRef {
        source_file: document.source_file.clone(),
        source: target.source.clone(),
        section_title: section.title.clone(),
        outline_path: section.outline_path.clone(),
        kind: target.kind,
        key: target.key.clone(),
        value: target.value.clone(),
    }
}

fn workspace_target_map(targets: &[WorkspaceTargetRef]) -> BTreeMap<String, Vec<usize>> {
    let mut target_map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, target) in targets.iter().enumerate() {
        target_map
            .entry(target.key.clone())
            .or_default()
            .push(index);
    }
    target_map
}

fn workspace_duplicate_issues(targets: &[WorkspaceTargetRef]) -> Vec<WorkspaceIssue> {
    let mut by_kind_and_key: BTreeMap<(TargetKindDiscriminant, String), Vec<&WorkspaceTargetRef>> =
        BTreeMap::new();
    for target in targets {
        let Some(kind) = duplicate_checked_kind(target.kind) else {
            continue;
        };
        by_kind_and_key
            .entry((kind, target.key.clone()))
            .or_default()
            .push(target);
    }

    let mut issues = Vec::new();
    for ((kind, key), duplicates) in by_kind_and_key {
        if duplicates.len() < 2 {
            continue;
        }
        for target in duplicates {
            issues.push(WorkspaceIssue {
                source_file: target.source_file.clone(),
                source: target.source.clone(),
                kind: match kind {
                    TargetKindDiscriminant::Id => {
                        WorkspaceIssueKind::DuplicateId { key: key.clone() }
                    }
                    TargetKindDiscriminant::CustomId => {
                        WorkspaceIssueKind::DuplicateCustomId { key: key.clone() }
                    }
                },
                message: format!(
                    "workspace target `{key}` is defined more than once; internal links are ambiguous"
                ),
            });
        }
    }
    issues
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TargetKindDiscriminant {
    Id,
    CustomId,
}

fn duplicate_checked_kind(kind: TargetKind) -> Option<TargetKindDiscriminant> {
    match kind {
        TargetKind::Id => Some(TargetKindDiscriminant::Id),
        TargetKind::CustomId => Some(TargetKindDiscriminant::CustomId),
        TargetKind::Headline
        | TargetKind::Target
        | TargetKind::RadioTarget
        | TargetKind::FootnoteDefinition
        | TargetKind::CodeRef => None,
    }
}

fn workspace_links(
    documents: &[WorkspaceDocument],
    target_map: &BTreeMap<String, Vec<usize>>,
    issues: &mut Vec<WorkspaceIssue>,
) -> Vec<WorkspaceLinkRef> {
    let targets = workspace_targets(documents);
    let mut links = Vec::new();
    for document in documents {
        for section in &document.sections {
            for link in &section.links {
                let key = link_resolution_key(&link.path);
                let resolved_target = key
                    .as_ref()
                    .and_then(|key| resolve_link_target(key, target_map, &targets));
                if let Some(key) = key.as_ref() {
                    collect_link_resolution_issue(
                        document.source_file.as_str(),
                        &link.source,
                        key,
                        target_map,
                        resolved_target.is_some(),
                        issues,
                    );
                }
                links.push(WorkspaceLinkRef {
                    source_file: document.source_file.clone(),
                    source: link.source.clone(),
                    section_title: section.title.clone(),
                    outline_path: section.outline_path.clone(),
                    path: link.path.clone(),
                    description: link.description.clone(),
                    search: link.search.clone(),
                    attachment: link.attachment.clone(),
                    file: link.file.clone(),
                    resolved_target,
                });
            }
        }
    }
    links
}

fn link_resolution_key(path: &str) -> Option<String> {
    let base = path
        .split_once("::")
        .map(|(base, _)| base)
        .unwrap_or(path)
        .trim();
    if base.is_empty() || is_external_like_link(base) {
        return None;
    }
    Some(base.to_string())
}

fn is_external_like_link(path: &str) -> bool {
    path.contains(':')
        && !(path.starts_with("id:") || path.starts_with("fn:") || path.starts_with("coderef:"))
}

fn resolve_link_target(
    key: &str,
    target_map: &BTreeMap<String, Vec<usize>>,
    targets: &[WorkspaceTargetRef],
) -> Option<WorkspaceResolvedTarget> {
    let indexes = target_map.get(key)?;
    if indexes.len() != 1 {
        return None;
    }
    let target = &targets[indexes[0]];
    Some(WorkspaceResolvedTarget {
        source_file: target.source_file.clone(),
        section_title: target.section_title.clone(),
        outline_path: target.outline_path.clone(),
        kind: target.kind,
        key: target.key.clone(),
        value: target.value.clone(),
    })
}

fn collect_link_resolution_issue(
    source_file: &str,
    source: &SectionIndexSource,
    key: &str,
    target_map: &BTreeMap<String, Vec<usize>>,
    resolved: bool,
    issues: &mut Vec<WorkspaceIssue>,
) {
    let Some(kind) = link_resolution_issue_kind(key, target_map, resolved) else {
        return;
    };
    issues.push(WorkspaceIssue {
        source_file: source_file.to_string(),
        source: source.clone(),
        kind,
        message: format!("workspace link `{key}` does not resolve to exactly one target"),
    });
}

fn link_resolution_issue_kind(
    key: &str,
    target_map: &BTreeMap<String, Vec<usize>>,
    resolved: bool,
) -> Option<WorkspaceIssueKind> {
    if matches!(target_map.get(key), Some(indexes) if indexes.len() > 1) {
        return Some(WorkspaceIssueKind::AmbiguousInternalLink {
            key: key.to_string(),
        });
    }
    if resolved {
        return None;
    }
    if key.starts_with("id:") {
        Some(WorkspaceIssueKind::UnresolvedIdLink {
            key: key.to_string(),
        })
    } else if key.starts_with('#') {
        Some(WorkspaceIssueKind::UnresolvedCustomIdLink {
            key: key.to_string(),
        })
    } else if key.starts_with("fn:") {
        Some(WorkspaceIssueKind::UnresolvedFootnoteLink {
            key: key.to_string(),
        })
    } else if key.starts_with("coderef:") {
        Some(WorkspaceIssueKind::UnresolvedCodeRefLink {
            key: key.to_string(),
        })
    } else {
        None
    }
}

fn workspace_attachments(documents: &[WorkspaceDocument]) -> Vec<WorkspaceAttachmentRef> {
    documents
        .iter()
        .flat_map(workspace_document_attachments)
        .collect()
}

fn workspace_document_attachments(document: &WorkspaceDocument) -> Vec<WorkspaceAttachmentRef> {
    document
        .sections
        .iter()
        .flat_map(|section| workspace_section_attachments(document, section))
        .collect()
}

fn workspace_section_attachments(
    document: &WorkspaceDocument,
    section: &SectionIndexRecord,
) -> Vec<WorkspaceAttachmentRef> {
    section_directory_attachment(document, section)
        .into_iter()
        .chain(section.links.iter().filter_map(|link| {
            link.attachment
                .as_ref()
                .map(|attachment| WorkspaceAttachmentRef {
                    source_file: document.source_file.clone(),
                    source: link.source.clone(),
                    section_title: section.title.clone(),
                    outline_path: section.outline_path.clone(),
                    kind: WorkspaceAttachmentKind::Link,
                    path: attachment.path.clone(),
                    link: Some(attachment.clone()),
                })
        }))
        .collect()
}

fn section_directory_attachment(
    document: &WorkspaceDocument,
    section: &SectionIndexRecord,
) -> Option<WorkspaceAttachmentRef> {
    section
        .attachment
        .directory
        .as_ref()
        .map(|directory| WorkspaceAttachmentRef {
            source_file: document.source_file.clone(),
            source: section.source.clone(),
            section_title: section.title.clone(),
            outline_path: section.outline_path.clone(),
            kind: WorkspaceAttachmentKind::SectionDirectory,
            path: directory.path.clone(),
            link: None,
        })
}
