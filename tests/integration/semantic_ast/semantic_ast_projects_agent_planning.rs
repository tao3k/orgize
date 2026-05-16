use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        AgendaDate, AgendaQuery, AgentPlanningDecision, AgentPlanningQuery, AgentPlanningSeverity,
    },
    Org,
};

const SOURCE: &str = r#"* TODO Late deadline
DEADLINE: <2026-05-10 Sun>
* TODO Deadline warning
DEADLINE: <2026-05-16 Sat -3d>
* TODO Delayed scheduled
SCHEDULED: <2026-05-14 Thu -2d>
* TODO Plain meeting
<2026-05-14 Thu 09:00-09:30>
"#;

#[test]
fn semantic_ast_projects_agent_planning_snapshot_from_agenda_rows() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let query = AgentPlanningQuery::new(AgendaQuery::new(
        AgendaDate::new(2026, 5, 14),
        AgendaDate::new(2026, 5, 16),
    ));
    let snapshot = doc.agent_planning_snapshot(&query);

    assert_eq!(snapshot.cards.len(), 6);
    let overdue = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Late deadline");
    let Some(overdue) = overdue else {
        panic!("overdue deadline card should exist");
    };
    assert_eq!(
        overdue.decision,
        AgentPlanningDecision::DeadlineOverdue { days_overdue: 4 }
    );
    assert_eq!(overdue.decision.severity(), AgentPlanningSeverity::Alert);
    assert_eq!(overdue.source.start.line, 1);

    let delayed = snapshot
        .cards
        .iter()
        .find(|card| card.title == "Delayed scheduled");
    let Some(delayed) = delayed else {
        panic!("delayed scheduled card should exist");
    };
    assert_eq!(
        delayed.decision,
        AgentPlanningDecision::ScheduledDelayed { days_delayed: 2 }
    );
    assert_eq!(delayed.display_date, AgendaDate::new(2026, 5, 16));

    let rendered = snapshot.to_compact_text("agent.org");
    assert!(rendered.contains("[PLAN001] Alert: Deadline overdue\n@ agent.org:1:1"));
    assert!(rendered.contains("[PLAN003] Action: Deadline warning\n@ agent.org:3:1"));
    assert!(rendered.contains("[PLAN005] Action: Scheduled task delayed"));
    assert!(rendered.contains("[PLAN007] Info: Active timestamp"));
    assert!(rendered.contains(
        "contract: Derived from official Org agenda syntax; no custom source syntax is required."
    ));
}
