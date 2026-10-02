use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        LifecycleRecordKind, MemoryEvidenceKind, MemoryLifecycleKind, MemoryQuery,
        MemoryRecordState,
    },
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
    assert!(
        matches!(
            &records[0].kind,
            LifecycleRecordKind::StateChange { timestamp, .. }
                if timestamp.as_deref() == Some("[2026-05-13 Wed]")
        ),
        "{:?}",
        records[0].kind
    );
    assert!(
        matches!(
            &records[1].kind,
            LifecycleRecordKind::Note { timestamp }
                if timestamp.as_deref() == Some("[2026-05-14 Thu]")
        ),
        "{:?}",
        records[1].kind
    );
    assert!(
        matches!(
            &records[2].kind,
            LifecycleRecordKind::Refile { timestamp, .. }
                if timestamp.as_deref() == Some("[2026-05-14 Thu]")
        ),
        "{:?}",
        records[2].kind
    );
    assert!(
        matches!(
            &records[3].kind,
            LifecycleRecordKind::Reschedule { from, to, timestamp }
                if from.as_deref() == Some("<2026-05-14 Thu>")
                    && to.as_deref() == Some("[2026-05-15 Fri]")
                    && timestamp.as_deref() == Some("[2026-05-15 Fri]")
        ),
        "{:?}",
        records[3].kind
    );
    assert!(
        matches!(
            &records[4].kind,
            LifecycleRecordKind::Redeadline { from, to, timestamp }
                if from.as_deref() == Some("<2026-05-16 Sat>")
                    && to.as_deref() == Some("[2026-05-15 Fri]")
                    && timestamp.as_deref() == Some("[2026-05-15 Fri]")
        ),
        "{:?}",
        records[4].kind
    );
    assert!(
        matches!(
            &records[5].kind,
            LifecycleRecordKind::Clock { timestamp, duration: Some(duration) }
                if timestamp.as_deref() == Some("[2026-05-14 Thu 10:00]")
                    && duration.total_seconds == 1_800
        ),
        "{:?}",
        records[5].kind
    );
    assert!(
        records
            .iter()
            .any(|record| matches!(record.kind, LifecycleRecordKind::StateChange { .. }))
    );
    assert!(
        records
            .iter()
            .any(|record| matches!(record.kind, LifecycleRecordKind::Refile { .. }))
    );
    assert!(records.iter().any(|record| matches!(
        &record.kind,
        LifecycleRecordKind::Refile {
            target: Some(target),
            ..
        } if target == "[[file:old.org][old]]"
    )));
    assert!(
        records
            .iter()
            .any(|record| matches!(record.kind, LifecycleRecordKind::Reschedule { .. }))
    );
    assert!(
        records
            .iter()
            .any(|record| matches!(record.kind, LifecycleRecordKind::Redeadline { .. }))
    );
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
fn logbook_records_use_aot_drawer_bounds_with_crlf() {
    let source = "* Work\r\n:LOGBOOK:\r\n- State \"DONE\" from \"TODO\" [2026-05-13 Wed]\r\n:LOGBOOK:\r\n:END:\r\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 2);
    assert!(matches!(
        records[0].kind,
        LifecycleRecordKind::StateChange { .. }
    ));
    assert_eq!(records[1].raw, ":LOGBOOK:");
    assert!(matches!(records[1].kind, LifecycleRecordKind::Note { .. }));
}

#[test]
fn logbook_kind_projection_preserves_alternate_prefixes_and_fallback() {
    let source = "* Work\n:LOGBOOK:\n- Refiling to [[file:notes.org]]\n- Deadline changed [2026-05-14 Thu]\n- Removed deadline [2026-05-15 Fri]\n- unclassified note\n:END:\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 4);
    assert!(matches!(
        records[0].kind,
        LifecycleRecordKind::Refile { .. }
    ));
    assert!(matches!(
        records[1].kind,
        LifecycleRecordKind::Redeadline { .. }
    ));
    assert!(matches!(
        records[2].kind,
        LifecycleRecordKind::Redeadline { .. }
    ));
    assert!(matches!(records[3].kind, LifecycleRecordKind::Note { .. }));
}

#[test]
fn logbook_refile_target_uses_first_aot_link_on_its_own_line() {
    let source = "* Work\r\n:LOGBOOK:\r\n- Refiled from [[file:α.org][alpha]] via [[file:later.org]]\r\n- Refiled again to [[file:next.org]]\r\n- Refiled without a link\r\n- Refiled with [[broken] text\r\n:END:\r\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 4);
    assert!(matches!(
        &records[0].kind,
        LifecycleRecordKind::Refile {
            target: Some(target),
            ..
        } if target == "[[file:α.org][alpha]]"
    ));
    assert!(matches!(
        &records[1].kind,
        LifecycleRecordKind::Refile {
            target: Some(target),
            ..
        } if target == "[[file:next.org]]"
    ));
    assert!(records[2..].iter().all(|record| matches!(
        record.kind,
        LifecycleRecordKind::Refile { target: None, .. }
    )));
}

#[test]
fn logbook_timestamp_ranges_and_invalid_clock_use_aot_facts() {
    let source = "* Work\n:LOGBOOK:\n- Rescheduled from [2026-05-14 Thu]--[2026-05-15 Fri] on [2026-05-16 Sat]\n- Note taken on [not-a-date]\nCLOCK: [2026-05-14 Thu 10:00] => tomorrow\n:END:\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 3);
    assert!(
        matches!(
            &records[0].kind,
            LifecycleRecordKind::Reschedule { from, to, timestamp }
                if from.as_deref() == Some("[2026-05-14 Thu]")
                    && to.as_deref() == Some("[2026-05-15 Fri]")
                    && timestamp.as_deref() == Some("[2026-05-16 Sat]")
        ),
        "{:?}",
        records[0].kind
    );
    assert!(
        matches!(
            &records[1].kind,
            LifecycleRecordKind::Note { timestamp: None }
        ),
        "{:?}",
        records[1].kind
    );
    assert!(
        matches!(
            &records[2].kind,
            LifecycleRecordKind::MalformedLogbook { .. }
        ),
        "{:?}",
        records[2].kind
    );
}

#[test]
fn logbook_list_item_clock_keeps_scheme_owned_duration_projection() {
    let source = "* Work\n:LOGBOOK:\n- CLOCK: [2026-05-14 Thu 10:00]--[2026-05-14 Thu 10:30] => 0:30\n:END:\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 1);
    assert!(
        matches!(
            &records[0].kind,
            LifecycleRecordKind::Clock { duration: Some(duration), timestamp }
                if duration.total_seconds == 1_800
                    && timestamp.as_deref() == Some("[2026-05-14 Thu 10:00]")
        ),
        "{:?}",
        records[0].kind
    );
}

#[test]
fn logbook_state_values_follow_scheme_aot_quote_boundaries() {
    let source = "* Work\n:LOGBOOK:\n- State \"DÖNE\" from \"TODO\" [2026-05-13 Wed]\n- State \"\" from \"\"\n- State \"DONE\" from \"TODO\n- State DONE from TODO\n:END:\n";
    let doc = Org::parse(source).document();
    assert_clean_projection(&doc);
    let records = doc.lifecycle_records();
    assert_eq!(records.len(), 4);
    assert!(matches!(
        &records[0].kind,
        LifecycleRecordKind::StateChange {
            to: Some(to),
            from: Some(from),
            ..
        } if to == "DÖNE" && from == "TODO"
    ));
    assert!(matches!(
        &records[1].kind,
        LifecycleRecordKind::StateChange {
            to: Some(to),
            from: Some(from),
            ..
        } if to.is_empty() && from.is_empty()
    ));
    assert!(
        records[2..]
            .iter()
            .all(|record| matches!(record.kind, LifecycleRecordKind::MalformedLogbook { .. }))
    );
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
    assert!(
        active
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::ArchiveProperty)
    );
    assert!(
        active.evidence.iter().any(|evidence| evidence.kind
            == MemoryEvidenceKind::Lifecycle(MemoryLifecycleKind::StateChange))
    );

    let archived = records
        .iter()
        .find(|record| record.title == "Archived subtree")
        .expect("archived memory");
    assert_eq!(archived.state, MemoryRecordState::Archived);
    assert!(
        archived
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::ArchiveTag)
    );
    assert!(
        archived
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::ArchiveLocation)
    );
}
