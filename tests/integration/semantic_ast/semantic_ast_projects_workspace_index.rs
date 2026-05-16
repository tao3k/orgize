use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{WorkspaceAttachmentKind, WorkspaceIndexBuilder, WorkspaceIssueKind},
    Org,
};

const WORKSPACE_A: &str = include_str!("../../fixtures/semantic_ast/workspace-index-a.org");
const WORKSPACE_B: &str = include_str!("../../fixtures/semantic_ast/workspace-index-b.org");

#[test]
fn semantic_ast_projects_workspace_index_from_document_local_records() {
    let doc_a = Org::parse(WORKSPACE_A).document();
    let doc_b = Org::parse(WORKSPACE_B).document();
    assert_clean_projection(&doc_a);
    assert_clean_projection(&doc_b);

    let mut builder = WorkspaceIndexBuilder::new();
    builder
        .add_document("workspace-a.org", &doc_a)
        .add_document("workspace-b.org", &doc_b);
    let index = builder.finish();

    assert_eq!(index.documents.len(), 2);
    assert_eq!(index.documents[0].summary.section_count, 2);
    assert_eq!(index.documents[0].summary.source_block_count, 1);
    assert!(index
        .targets
        .iter()
        .any(|target| target.source_file == "workspace-a.org" && target.key == "id:alpha-id"));
    assert!(index
        .targets
        .iter()
        .any(|target| target.source_file == "workspace-b.org" && target.key == "#beta-custom"));

    let beta_link = index
        .links
        .iter()
        .find(|link| link.path == "id:beta-id::*Beta")
        .expect("beta id link");
    let resolved_beta = beta_link
        .resolved_target
        .as_ref()
        .expect("beta id link resolves across files");
    assert_eq!(resolved_beta.source_file, "workspace-b.org");
    assert_eq!(resolved_beta.key, "id:beta-id");

    assert!(index.attachments.iter().any(|attachment| {
        attachment.kind == WorkspaceAttachmentKind::SectionDirectory
            && attachment.path == "alpha-assets"
    }));
    assert!(index.attachments.iter().any(|attachment| {
        attachment.kind == WorkspaceAttachmentKind::Link && attachment.path == "plan.pdf"
    }));

    assert!(index.issues.iter().any(|issue| {
        matches!(
            &issue.kind,
            WorkspaceIssueKind::DuplicateId { key } if key == "id:shared-id"
        )
    }));
    assert!(index.issues.iter().any(|issue| {
        matches!(
            &issue.kind,
            WorkspaceIssueKind::AmbiguousInternalLink { key } if key == "id:shared-id"
        )
    }));
    assert!(index.issues.iter().any(|issue| {
        matches!(
            &issue.kind,
            WorkspaceIssueKind::UnresolvedIdLink { key } if key == "id:missing-id"
        )
    }));

    insta::assert_debug_snapshot!("semantic_ast__semantic_workspace_index", index);
}

#[test]
fn semantic_ast_builds_single_document_workspace_index() {
    let doc = Org::parse(WORKSPACE_A).document();
    assert_clean_projection(&doc);

    let index = doc.workspace_index("workspace-a.org");
    assert_eq!(index.documents.len(), 1);
    assert!(index.links.iter().any(|link| link.path == "id:missing-id"));
}
