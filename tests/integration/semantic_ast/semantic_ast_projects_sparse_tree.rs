use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{SparseTreeMatchKind, SparseTreeQuery, SparseTreeReceiptKind, SparseTreeSkipReason},
};

const SOURCE: &str = r#"#+CATEGORY: memory
* TODO [#A] Agent memory :agent:
:PROPERTIES:
:ID: active-memory
:PREF: org native projection
:END:
The current memory says use sparse tree cards for corrected facts.
See [[id:archived-memory::*Retired memory][old memory]] and <<active-target>>.
** DONE Retired memory :agent:ARCHIVE:
CLOSED: [2026-05-12 Tue]
:PROPERTIES:
:ID: archived-memory
:END:
This retired memory should remain searchable evidence, but not active authority.
* TODO Planning card :plan:
SCHEDULED: <2026-05-15 Fri>
"#;

#[test]
fn semantic_ast_projects_sparse_tree_cards_from_org_match_and_text() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let query = SparseTreeQuery::new()
        .source_file("memory.org")
        .text("corrected")
        .match_expression(r#"+agent+TODO="TODO"+PRIORITY="A""#)
        .expect("valid agenda match expression");
    let projection = doc.sparse_tree_projection(&query);

    assert_eq!(projection.cards.len(), 1);
    let card = &projection.cards[0];
    assert_eq!(card.title, "Agent memory");
    assert_eq!(card.outline_path, ["Agent memory"]);
    assert_eq!(
        card.todo.as_ref().map(|todo| todo.name.as_str()),
        Some("TODO")
    );
    assert_eq!(card.priority.effective_text(), "A");
    assert_eq!(card.effective_tags, ["agent"]);
    assert!(
        card.preview
            .as_ref()
            .is_some_and(|preview| preview.contains("sparse tree cards"))
    );
    assert!(
        card.matches
            .iter()
            .any(|matched| matched.kind == SparseTreeMatchKind::Tag && matched.value == "agent")
    );
    assert!(card.matches.iter().any(|matched| {
        matched.kind == SparseTreeMatchKind::SpecialProperty
            && matched.key.as_deref() == Some("TODO")
            && matched.value == "TODO"
    }));
    assert!(card.matches.iter().any(|matched| {
        matched.kind == SparseTreeMatchKind::Priority
            && matched.key.as_deref() == Some("PRIORITY")
            && matched.value == "A"
    }));
    assert!(
        card.matches
            .iter()
            .any(|matched| matched.kind == SparseTreeMatchKind::Body
                && matched.value.contains("corrected facts"))
    );
    assert!(
        card.receipts
            .iter()
            .any(|receipt| receipt.kind == SparseTreeReceiptKind::MatchExpressionMatched)
    );
    assert!(
        card.receipts
            .iter()
            .any(|receipt| receipt.kind == SparseTreeReceiptKind::TextMatched)
    );
    assert!(
        card.receipts
            .iter()
            .any(|receipt| receipt.kind == SparseTreeReceiptKind::Accepted)
    );
    assert!(
        card.links
            .iter()
            .any(|link| link.path == "id:archived-memory::*Retired memory")
    );
    assert!(
        card.targets
            .iter()
            .any(|target| target.key == "id:active-memory")
    );
}

#[test]
fn semantic_ast_sparse_tree_preserves_archive_evidence_but_can_filter_it() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let query = SparseTreeQuery::new().text("retired memory");
    let projection = doc.sparse_tree_projection(&query);
    assert!(
        projection
            .cards
            .iter()
            .any(|card| card.title == "Retired memory" && card.archive.archived)
    );

    let active_query = SparseTreeQuery::new()
        .include_archived(false)
        .text("retired memory")
        .explain_skips(true);
    let active_projection = doc.sparse_tree_projection(&active_query);
    assert!(
        active_projection
            .cards
            .iter()
            .all(|card| card.title != "Retired memory")
    );
    assert!(active_projection.skipped.iter().any(|skip| {
        skip.title == "Retired memory" && skip.reason == SparseTreeSkipReason::Archived
    }));
    assert!(active_projection.skipped.iter().any(|skip| {
        skip.receipts
            .iter()
            .any(|receipt| receipt.kind == SparseTreeReceiptKind::SkippedArchived)
    }));
}

#[test]
fn semantic_ast_sparse_tree_renders_compact_agent_snapshot() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let query = SparseTreeQuery::new().text("old memory");
    let projection = doc.sparse_tree_projection(&query);
    let rendered = projection.to_compact_text("memory.org");

    assert!(rendered.contains("[SPARSE001] Match: Agent memory\n@ memory.org:2:1"));
    assert!(rendered.contains("outline: Agent memory"));
    assert!(rendered.contains("matches:"));
    assert!(rendered.contains("receipt: candidate,visibilityFilterPassed,textMatched,accepted"));
    assert!(rendered.contains("body=The current memory says use sparse tree cards"));
    assert!(rendered.contains("links: id:archived-memory::*Retired memory"));
    assert!(rendered.contains(
        "contract: Derived from official Org sparse-tree/search constructs; no custom source syntax is required."
    ));
}
