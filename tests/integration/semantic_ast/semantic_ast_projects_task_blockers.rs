use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{AgendaDate, AgendaQuery, AgendaViewQuery, AgentPlanningQuery, TaskBlockerKind},
    Org,
};

const SOURCE: &str = r#"* TODO Project
:PROPERTIES:
:ORDERED: t
:END:
** TODO First
SCHEDULED: <2026-05-15 Fri>
*** TODO Nested A
*** TODO Nested B
** DONE Done sibling
** TODO Second
SCHEDULED: <2026-05-15 Fri>
** TODO Third
SCHEDULED: <2026-05-15 Fri>
"#;

#[test]
fn semantic_ast_projects_task_blockers_follow_local_ordered_siblings() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let blockers = doc.task_blocker_records();
    assert_eq!(blockers.len(), 2);

    let second = blockers
        .iter()
        .find(|record| record.blocked.title == "Second")
        .expect("Second should be blocked by the first open sibling");
    assert_eq!(second.kind, TaskBlockerKind::OrderedPreviousSibling);
    assert_eq!(second.blocker.title, "First");
    assert_eq!(second.parent.title, "Project");
    assert_eq!(second.parent.outline_path, vec!["Project"]);
    assert_eq!(second.parent.ordered_property_source.start.line, 3);
    assert!(second.message.contains("local ORDERED property"));

    let third = blockers
        .iter()
        .find(|record| record.blocked.title == "Third")
        .expect("Third should be blocked by the nearest previous open sibling");
    assert_eq!(third.blocker.title, "Second");

    assert!(!blockers
        .iter()
        .any(|record| record.blocked.title == "Nested B"));
}

#[test]
fn semantic_ast_projects_agenda_view_embeds_ordered_sibling_blockers() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let plan = doc.agenda_view_plan(&AgendaViewQuery::single_day(AgendaDate::new(2026, 5, 15)));
    let second = plan
        .cards
        .iter()
        .find(|card| card.title == "Second")
        .expect("Second agenda card should exist");

    assert_eq!(second.blockers.len(), 1);
    assert_eq!(second.blockers[0].blocker.title, "First");
    assert!(second
        .receipts
        .iter()
        .any(|receipt| receipt.kind.as_str() == "blockedByOrderedSibling"));

    let rendered = plan.to_compact_text("ordered.org");
    assert!(rendered.contains("[AGENDA_ACCEPT] Second"));
    assert!(rendered.contains("blocked-by: orderedPreviousSibling First @ 5:1"));
}

#[test]
fn semantic_ast_projects_agent_planning_embeds_ordered_sibling_blockers() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let query = AgentPlanningQuery::new(AgendaQuery::single_day(AgendaDate::new(2026, 5, 15)));
    let snapshot = doc.agent_planning_snapshot(&query);
    let second = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Second")
        .expect("Second planning card should exist");

    assert_eq!(second.blockers.len(), 1);
    assert_eq!(second.blockers[0].blocker.title, "First");
    assert!(snapshot
        .to_compact_text("ordered.org")
        .contains("blocked-by: orderedPreviousSibling First @ 5:1"));
}
