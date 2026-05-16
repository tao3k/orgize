use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        LifecycleRecordKind, MemoryEvidenceKind, MemoryLifecycleKind, MemoryQuery,
        MemoryRecordState,
    },
    Org,
};

const SOURCE: &str = r#"#+ARCHIVE: archive.org::* Archived
* TODO Active with archive location :work:
:PROPERTIES:
:ARCHIVE: tasks.org::* Finished tasks
:END:
:LOGBOOK:
- State "WAIT" from "TODO" [2026-05-13 Wed]
- Note taken on [2026-05-14 Thu]
- Refiled on [2026-05-14 Thu] from [[file:old.org][old]]
- Rescheduled from "<2026-05-14 Thu>" on [2026-05-15 Fri]
- New deadline from "<2026-05-16 Sat>" on [2026-05-15 Fri]
CLOCK: [2026-05-14 Thu 10:00]--[2026-05-14 Thu 10:30] =>  0:30
:END:
* TODO Archived subtree :work:ARCHIVE:
"#;

#[test]
fn semantic_ast_projects_lifecycle_and_archive_metadata() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let archive = &doc.archive_locations[0];
    assert_eq!(archive.value, "archive.org::* Archived");
    assert_eq!(archive.file.as_deref(), Some("archive.org"));
    assert_eq!(archive.heading.as_deref(), Some("* Archived"));

    let active = &doc.sections[0];
    assert!(!active.archive.archived);
    let active_location = active
        .archive
        .location()
        .expect("archive property location");
    assert_eq!(active_location.file.as_deref(), Some("tasks.org"));
    assert_eq!(active_location.heading.as_deref(), Some("* Finished tasks"));

    let archived = &doc.sections[1];
    assert!(archived.archive.archived);
    assert!(archived.archive.has_archive_tag);
    assert_eq!(
        archived
            .archive
            .location()
            .expect("keyword archive location")
            .file
            .as_deref(),
        Some("archive.org")
    );

    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 6);
    assert!(records
        .iter()
        .any(|record| matches!(record.kind, LifecycleRecordKind::StateChange { .. })));
    assert!(records
        .iter()
        .any(|record| matches!(record.kind, LifecycleRecordKind::Refile { .. })));
    assert!(records
        .iter()
        .any(|record| matches!(record.kind, LifecycleRecordKind::Reschedule { .. })));
    assert!(records
        .iter()
        .any(|record| matches!(record.kind, LifecycleRecordKind::Redeadline { .. })));
    assert!(records.iter().any(|record| {
        matches!(
            &record.kind,
            LifecycleRecordKind::Clock {
                duration: Some(duration),
                ..
            } if duration.total_seconds == 1_800
        )
    }));

    let compact = records
        .iter()
        .map(|record| {
            (
                record.section_title.as_str(),
                record.kind.title(),
                record.raw.as_str(),
            )
        })
        .collect::<Vec<_>>();
    insta::assert_debug_snapshot!("semantic_ast__semantic_lifecycle_records", compact);
}

#[test]
fn semantic_ast_projects_memory_uses_lifecycle_and_archive_evidence() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.memory_records(&MemoryQuery::new().require_tag("work"));
    let active = records
        .iter()
        .find(|record| record.title == "Active with archive location")
        .expect("active memory");
    assert_eq!(active.state, MemoryRecordState::Current);
    assert!(active
        .evidence
        .iter()
        .any(|evidence| evidence.kind == MemoryEvidenceKind::ArchiveProperty));
    assert!(active
        .evidence
        .iter()
        .any(|evidence| evidence.kind
            == MemoryEvidenceKind::Lifecycle(MemoryLifecycleKind::StateChange)));

    let archived = records
        .iter()
        .find(|record| record.title == "Archived subtree")
        .expect("archived memory");
    assert_eq!(archived.state, MemoryRecordState::Archived);
    assert!(archived
        .evidence
        .iter()
        .any(|evidence| evidence.kind == MemoryEvidenceKind::ArchiveTag));
    assert!(archived
        .evidence
        .iter()
        .any(|evidence| evidence.kind == MemoryEvidenceKind::ArchiveLocation));
}
