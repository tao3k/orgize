//! Source-owned edit validation for consumers such as Lambda Aitia.

use orgize::org_aot_edit::{
    OrgSourceEdit, OrgSourceEditError, apply_org_source_edits, org_source_digest,
};

const SOURCE: &str = "* Design\n:PROPERTIES:\n:ID: design-1\n:END:\nOld guarantee\n* Notes\n:PROPERTIES:\n:ID: notes-1\n:END:\nKeep me\n";

fn edit<'a>(
    source: &str,
    node_id: &'a str,
    text: &'a str,
    replacement: &'a str,
) -> OrgSourceEdit<'a> {
    let start_byte = source.find(text).expect("fixture text");
    OrgSourceEdit {
        node_id,
        start_byte,
        end_byte: start_byte + text.len(),
        expected_old: text,
        replacement,
    }
}

#[test]
fn source_owned_edit_preserves_unrelated_org_bytes() {
    let candidate = apply_org_source_edits(
        SOURCE,
        &org_source_digest(SOURCE),
        &[edit(SOURCE, "design-1", "Old guarantee", "New guarantee")],
    )
    .expect("valid owner-bound edit");
    assert_eq!(candidate, SOURCE.replace("Old guarantee", "New guarantee"));
    assert!(candidate.contains(":ID: notes-1\n:END:\nKeep me\n"));
}

#[test]
fn rejects_stale_wrong_node_and_changed_span() {
    let correct = edit(SOURCE, "design-1", "Old guarantee", "New guarantee");
    assert!(matches!(
        apply_org_source_edits(
            SOURCE,
            &org_source_digest("old source"),
            std::slice::from_ref(&correct)
        ),
        Err(OrgSourceEditError::StaleSource)
    ));
    assert!(matches!(
        apply_org_source_edits(
            SOURCE,
            &org_source_digest(SOURCE),
            &[edit(SOURCE, "notes-1", "Old guarantee", "New guarantee")]
        ),
        Err(OrgSourceEditError::WrongNode)
    ));
    let changed = OrgSourceEdit {
        expected_old: "not old",
        ..correct
    };
    assert!(matches!(
        apply_org_source_edits(SOURCE, &org_source_digest(SOURCE), &[changed]),
        Err(OrgSourceEditError::ChangedContent)
    ));
}

#[test]
fn rejects_overlaps_duplicate_identity_and_non_utf8_boundary() {
    let first = edit(SOURCE, "design-1", "Old guarantee", "New guarantee");
    let overlap = OrgSourceEdit {
        node_id: "notes-1",
        start_byte: first.start_byte + 1,
        end_byte: first.end_byte,
        expected_old: "ld guarantee",
        replacement: "other",
    };
    assert!(matches!(
        apply_org_source_edits(
            SOURCE,
            &org_source_digest(SOURCE),
            &[first.clone(), overlap]
        ),
        Err(OrgSourceEditError::Overlap)
    ));
    assert!(matches!(
        apply_org_source_edits(SOURCE, &org_source_digest(SOURCE), &[first.clone(), first]),
        Err(OrgSourceEditError::DuplicateNodeId)
    ));

    let unicode = "* 设计\n:PROPERTIES:\n:ID: design-1\n:END:\n说明\n";
    let split = unicode.find("说明").unwrap() + 1;
    let bad = OrgSourceEdit {
        node_id: "design-1",
        start_byte: split,
        end_byte: split + 1,
        expected_old: "",
        replacement: "x",
    };
    assert!(matches!(
        apply_org_source_edits(unicode, &org_source_digest(unicode), &[bad]),
        Err(OrgSourceEditError::InvalidRange)
    ));
}

#[test]
fn rejects_missing_ambiguous_and_removed_node_identity() {
    assert!(matches!(
        apply_org_source_edits(SOURCE, &org_source_digest(SOURCE), &[]),
        Err(OrgSourceEditError::EmptyEdits)
    ));
    assert!(matches!(
        apply_org_source_edits(
            SOURCE,
            &org_source_digest(SOURCE),
            &[edit(SOURCE, "absent", "Old guarantee", "New guarantee")]
        ),
        Err(OrgSourceEditError::MissingNodeId)
    ));
    let duplicate = SOURCE.replace(":ID: notes-1", ":ID: design-1");
    assert!(matches!(
        apply_org_source_edits(
            &duplicate,
            &org_source_digest(&duplicate),
            &[edit(
                &duplicate,
                "design-1",
                "Old guarantee",
                "New guarantee"
            )]
        ),
        Err(OrgSourceEditError::AmbiguousNodeId)
    ));
    assert!(matches!(
        apply_org_source_edits(
            SOURCE,
            &org_source_digest(SOURCE),
            &[edit(SOURCE, "design-1", "design-1", "renamed")]
        ),
        Err(OrgSourceEditError::MissingNodeId)
    ));
}
