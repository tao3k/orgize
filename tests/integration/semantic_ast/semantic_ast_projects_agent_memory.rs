use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        AgentMemoryDecision, AgentMemoryQuery, AgentMemorySeverity, MemoryEvidenceKind,
        MemoryQuery, MemoryRecordState,
    },
    Org,
};

const SOURCE: &str = r#"* TODO Current preference :agent:
:PROPERTIES:
:ID: mem-current
:PREF: use Org-native projection
:END:
The current agent note links to [[id:mem-old][the corrected record]] on <2026-05-14 Thu>.
* DONE Old preference :agent:
CLOSED: [2026-05-10 Sun]
:LOGBOOK:
- State "DONE" from "TODO" [2026-05-10 Sun]
CLOCK: [2026-05-10 Sun 09:00]--[2026-05-10 Sun 09:30] =>  0:30
:END:
This old note should remain visible as history.
* TODO Archived preference :agent:ARCHIVE:
This archived note should not be promoted as active.
* Research context :agent:
Background note without task lifecycle.
"#;

#[test]
fn semantic_ast_projects_agent_memory_records_from_org_constructs() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.memory_records(&MemoryQuery::new().require_tag("agent"));
    assert_eq!(records.len(), 4);

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
    assert!(current
        .evidence
        .iter()
        .any(|evidence| matches!(evidence.kind, MemoryEvidenceKind::Timestamp { .. })));

    let old = records
        .iter()
        .find(|record| record.title == "Old preference")
        .expect("old memory record");
    assert_eq!(old.state, MemoryRecordState::Closed);
    assert!(old
        .evidence
        .iter()
        .any(|evidence| evidence.kind == MemoryEvidenceKind::Closed));
    assert!(old
        .evidence
        .iter()
        .any(|evidence| evidence.kind == MemoryEvidenceKind::Logbook));
    assert!(old
        .evidence
        .iter()
        .any(|evidence| evidence.kind == MemoryEvidenceKind::Clock));

    let archived = records
        .iter()
        .find(|record| record.title == "Archived preference")
        .expect("archived memory record");
    assert_eq!(archived.state, MemoryRecordState::Archived);

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
    assert_eq!(snapshot.cards.len(), 4);

    let current = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Current preference")
        .expect("current card");
    assert_eq!(current.decision, AgentMemoryDecision::Current);
    assert_eq!(current.decision.severity(), AgentMemorySeverity::Action);

    let old = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Old preference")
        .expect("old card");
    assert_eq!(old.decision, AgentMemoryDecision::Closed);
    assert_eq!(old.decision.severity(), AgentMemorySeverity::Suppressed);

    let rendered = snapshot.to_compact_text("memory.org");
    assert!(rendered.contains("[MEM001] Action: Current memory\n@ memory.org:1:1"));
    assert!(rendered.contains("[MEM002] Suppressed: Closed memory\n@ memory.org:7:1"));
    assert!(rendered.contains("[MEM003] Suppressed: Archived memory"));
    assert!(rendered.contains("[MEM004] Info: Background memory"));
    assert!(rendered.contains("links: id:mem-old"));
    assert!(rendered.contains(
        "contract: Derived from official Org memory-bearing constructs; no custom source syntax is required."
    ));
}
