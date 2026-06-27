use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{
        AgentMemoryDecision, AgentMemoryQuery, AgentMemorySeverity, MemoryAuthorityKind,
        MemoryEvidenceKind, MemoryQuery, MemoryRecordState,
    },
    document::{DocumentWalkConfig, OrgMemorySearchOptions, query_org_memory_records},
};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SOURCE: &str = r#"* TODO Current preference :agent:
:PROPERTIES:
:ID: mem-current
:PREF: use Org-native projection
:END:
The current agent note links to [[id:mem-old][the corrected record]] on <2026-05-14 Thu>.
* DONE Current preference :agent:
CLOSED: [2026-05-12 Tue]
This older version shares the current title and should stay historical.
* DONE Old preference :agent:
CLOSED: [2026-05-10 Sun]
:LOGBOOK:
- State "DONE" from "TODO" [2026-05-10 Sun]
CLOCK: [2026-05-10 Sun 09:00]--[2026-05-10 Sun 09:30] =>  0:30
:END:
This old note should remain visible as history.
* TODO Archived preference :agent:ARCHIVE:
:PROPERTIES:
:ARCHIVE_TIME: 2026-05-11 Mon 10:00
:END:
This archived note should not be promoted as active.
* TODO Daily habit :agent:
SCHEDULED: <2026-05-14 Thu +1d>
:PROPERTIES:
:STYLE: habit
:LAST_REPEAT: [2026-05-13 Wed]
:END:
* Research context :agent:
Background note without task lifecycle.
"#;

#[test]
fn semantic_ast_projects_agent_memory_records_from_org_constructs() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.memory_records(&MemoryQuery::new().require_tag("agent"));
    assert_eq!(records.len(), 6);

    let current = records
        .iter()
        .find(|record| record.title == "Current preference")
        .expect("current memory record");
    assert_eq!(current.state, MemoryRecordState::Current);
    assert!(current
        .properties
        .iter()
        .any(|property| property.key == "PREF" && property.value == "use Org-native projection"));
    assert!(current.links.iter().any(|link| link.path == "id:mem-old"));
    assert!(
        current
            .evidence
            .iter()
            .any(|evidence| matches!(evidence.kind, MemoryEvidenceKind::Timestamp { .. }))
    );
    assert!(current.evidence.iter().any(|evidence| {
        matches!(
            &evidence.kind,
            MemoryEvidenceKind::Identity { key } if key == "ID"
        )
    }));

    let old = records
        .iter()
        .find(|record| record.title == "Old preference")
        .expect("old memory record");
    assert_eq!(old.state, MemoryRecordState::Closed);
    assert!(
        old.evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::Closed)
    );
    assert!(
        old.evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::Logbook)
    );
    assert!(
        old.evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::Clock)
    );

    let superseded = records
        .iter()
        .find(|record| {
            record.title == "Current preference" && record.state == MemoryRecordState::Closed
        })
        .expect("superseded memory record");
    assert!(
        superseded
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::Closed)
    );

    let archived = records
        .iter()
        .find(|record| record.title == "Archived preference")
        .expect("archived memory record");
    assert_eq!(archived.state, MemoryRecordState::Archived);
    assert!(archived.evidence.iter().any(|evidence| evidence.kind
        == MemoryEvidenceKind::ArchiveProperty
        && evidence.value == "2026-05-11 Mon 10:00"));

    let habit = records
        .iter()
        .find(|record| record.title == "Daily habit")
        .expect("habit memory record");
    assert_eq!(habit.state, MemoryRecordState::Current);
    assert!(
        habit
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::HabitStyle)
    );
    assert!(
        habit
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::HabitLastRepeat)
    );
    assert!(
        habit
            .evidence
            .iter()
            .any(|evidence| evidence.kind == MemoryEvidenceKind::HabitRepeater)
    );

    let background = records
        .iter()
        .find(|record| record.title == "Research context")
        .expect("background memory record");
    assert_eq!(background.state, MemoryRecordState::Background);
}

#[test]
fn semantic_ast_renders_agent_memory_snapshot_as_compact_cards() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let snapshot = doc.agent_memory_snapshot(&AgentMemoryQuery::new(
        MemoryQuery::new().require_tag("agent"),
    ));
    assert_eq!(snapshot.cards.len(), 6);

    let current = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Current preference")
        .expect("current card");
    assert_eq!(current.decision, AgentMemoryDecision::Current);
    assert_eq!(current.decision.severity(), AgentMemorySeverity::Action);
    assert!(
        current
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Current)
    );
    assert!(
        current
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Identity)
    );
    assert!(
        current
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Temporal)
    );

    let old = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Old preference")
        .expect("old card");
    assert_eq!(old.decision, AgentMemoryDecision::Closed);
    assert_eq!(old.decision.severity(), AgentMemorySeverity::Suppressed);
    assert!(
        old.authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Closed)
    );
    assert!(
        old.authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Lifecycle)
    );

    let superseded = snapshot
        .cards
        .iter()
        .find(|card| {
            card.title == "Current preference" && card.decision == AgentMemoryDecision::Closed
        })
        .expect("superseded card");
    assert!(
        superseded
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::StaleCandidate)
    );
    assert!(
        superseded
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::SupersededCandidate)
    );

    let habit = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Daily habit")
        .expect("habit card");
    assert!(
        habit
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Habit)
    );
    assert!(
        habit
            .authority
            .iter()
            .any(|reason| reason.kind == MemoryAuthorityKind::Repeat)
    );

    let rendered = snapshot.to_compact_text("memory.org");
    assert!(rendered.contains("[MEM001] Action: Current memory\n@ memory.org:1:1"));
    assert!(rendered.contains("[MEM002] Suppressed: Closed memory\n@ memory.org:7:1"));
    assert!(
        rendered.contains(
            "a current memory card with the same anchor or title may supersede this fact"
        )
    );
    assert!(rendered.contains("[MEM003] Suppressed: Archived memory"));
    assert!(rendered.contains("[MEM004] Info: Background memory"));
    assert!(rendered.contains("habit evidence marks recurring cadence instead of a one-off fact"));
    assert!(
        rendered.contains("stable identity evidence lets agents correlate corrections across time")
    );
    assert!(rendered.contains("timestamp evidence gives this fact a bounded time context"));
    assert!(
        rendered.contains(
            "repeat evidence should be interpreted as cadence, not a timeless preference"
        )
    );
    assert!(rendered.contains("links: id:mem-old"));
    assert!(rendered.contains(
        "contract: Derived from official Org memory-bearing constructs; no custom source syntax is required."
    ));
}

#[test]
fn plan_ledger_memory_projection_stays_in_millisecond_budget() {
    let root = temp_test_dir("orgize-plan-ledger-projection-gate");
    let artifacts = root.join("artifacts").join("org");
    let plans = artifacts.join("flow").join("plans");
    fs::create_dir_all(&plans).expect("create plans dir");
    for index in 0..2_000 {
        let plan_id = if index == 777 {
            "memory-engine-hot-path".to_string()
        } else {
            format!("noise-plan-{index:05}")
        };
        fs::write(
            plans.join(format!("agent-plan-{plan_id}.org")),
            format!(
                "* TODO Plan {index} [1/8] [12%] :agent:plan:\n\
                 :PROPERTIES:\n\
                 :CONTRACT_ORG: agent.plan.v1\n\
                 :ID: {plan_id}\n\
                 :PLAN_ID: {plan_id}\n\
                 :PLAN_SESSION: session-a\n\
                 :OBJECTIVE: Stabilize memory engine recall flow {index}\n\
                 :NEXT_ACTION: continue checkpoint {index}\n\
                 :END:\n\
                 ** Evidence\n\
                 - receipt {index}\n",
            ),
        )
        .expect("write plan");
    }

    let (elapsed, records) = (0..5)
        .map(|_| {
            let started_at = Instant::now();
            let records = query_org_memory_records(
                &artifacts,
                &DocumentWalkConfig::default(),
                &OrgMemorySearchOptions::plan_ledgers(),
            )
            .expect("query plan ledgers");
            (started_at.elapsed(), records)
        })
        .min_by_key(|(elapsed, _)| *elapsed)
        .expect("plan ledger projection benchmark sample");

    assert_eq!(records.len(), 2_000);
    assert!(records.iter().any(
        |record| record.properties.get("PLAN_ID").map(String::as_str)
            == Some("memory-engine-hot-path")
    ));
    assert!(
        elapsed < Duration::from_millis(100),
        "plan ledger projection exceeded 100ms gate: {elapsed:?}"
    );
    let _ = fs::remove_dir_all(root);
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
