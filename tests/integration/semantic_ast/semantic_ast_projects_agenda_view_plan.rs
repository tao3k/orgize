use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        AgendaBlockViewQuery, AgendaDate, AgendaQuery, AgendaViewQuery, AgendaViewReceiptKind,
        AgendaViewSkipReason, AgendaViewSortKey, AgendaViewSortSpec,
    },
    Org,
};

#[test]
fn semantic_ast_projects_agenda_view_plan_receipts() {
    let doc = Org::parse(
        r#"* TODO Morning
SCHEDULED: <2026-05-15 Fri 09:00>
* TODO Deadline
DEADLINE: <2026-05-15 Fri>
* TODO Afternoon
SCHEDULED: <2026-05-15 Fri 13:00>
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let plan =
        doc.agenda_view_plan(&AgendaViewQuery::single_day(AgendaDate::new(2026, 5, 15)).limit(2));

    assert_eq!(plan.total_candidates, 3);
    assert_eq!(plan.limit, Some(2));
    assert_eq!(plan.cards.len(), 2);
    assert_eq!(plan.skipped.len(), 1);
    assert_eq!(plan.cards[0].title, "Deadline");
    assert_eq!(plan.cards[0].sorted_position, 1);
    assert!(plan.cards[0]
        .sort_keys
        .iter()
        .any(|sort_key| sort_key.key.as_str() == "kind" && sort_key.value == "deadline"));
    assert!(plan.cards[0]
        .receipts
        .iter()
        .any(|receipt| receipt.kind == AgendaViewReceiptKind::QueryMatched));
    assert!(plan.cards[0]
        .receipts
        .iter()
        .any(|receipt| receipt.kind == AgendaViewReceiptKind::Accepted));

    let skipped = &plan.skipped[0];
    assert_eq!(skipped.title, "Afternoon");
    assert_eq!(skipped.sorted_position, 3);
    assert!(matches!(
        skipped.reason,
        AgendaViewSkipReason::Limit { limit: 2 }
    ));
    assert!(skipped.receipts.iter().any(|receipt| receipt.kind
        == AgendaViewReceiptKind::SkippedLimit
        && receipt.message.contains("exceeds limit 2")));
}

#[test]
fn semantic_ast_projects_agenda_view_plan_applies_sort_strategy_subset() {
    let doc = Org::parse(
        r#"#+TODO: TODO WAITING | DONE
* TODO [#C] Low timed
SCHEDULED: <2026-05-15 Fri 09:00>
* WAITING [#A] High timed
SCHEDULED: <2026-05-15 Fri 09:00>
* TODO Untimed deadline
DEADLINE: <2026-05-15 Fri>
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let query = AgendaViewQuery::single_day(AgendaDate::new(2026, 5, 15))
        .sort_strategy([
            AgendaViewSortSpec::up(AgendaViewSortKey::Time),
            AgendaViewSortSpec::down(AgendaViewSortKey::Priority),
        ])
        .limit(2);
    let plan = doc.agenda_view_plan(&query);

    assert_eq!(plan.total_candidates, 3);
    assert_eq!(
        plan.sort_strategy,
        vec![
            AgendaViewSortSpec::up(AgendaViewSortKey::Time),
            AgendaViewSortSpec::down(AgendaViewSortKey::Priority),
        ]
    );
    assert_eq!(plan.cards.len(), 2);
    assert_eq!(plan.cards[0].title, "High timed");
    assert_eq!(plan.cards[1].title, "Low timed");
    assert_eq!(plan.skipped[0].title, "Untimed deadline");
    assert!(plan.cards[0].receipts.iter().any(|receipt| receipt.kind
        == AgendaViewReceiptKind::Sorted
        && receipt
            .message
            .contains("agenda sort strategy: time-up,priority-down")));
    assert!(plan.cards[0]
        .sort_keys
        .iter()
        .any(|sort_key| sort_key.key == AgendaViewSortKey::Priority && sort_key.value == "A"));
}

#[test]
fn semantic_ast_projects_agenda_block_view_plan_groups_named_sections() {
    let doc = Org::parse(
        r#"#+TODO: TODO WAITING | DONE
* TODO Ship patch
SCHEDULED: <2026-05-15 Fri 11:00>
* WAITING Review queue
SCHEDULED: <2026-05-15 Fri 09:00>
* TODO Later
SCHEDULED: <2026-05-16 Sat>
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let date = AgendaDate::new(2026, 5, 15);
    let waiting_agenda = AgendaQuery::single_day(date)
        .match_expression(r#"TODO="WAITING""#)
        .expect("valid TODO match");
    let block = doc.agenda_block_view_plan(
        &AgendaBlockViewQuery::new("Daily agent agenda")
            .section(
                "Timed today",
                AgendaViewQuery::single_day(date)
                    .sort_by(AgendaViewSortSpec::up(AgendaViewSortKey::Time)),
            )
            .section(
                "Waiting",
                AgendaViewQuery::new(waiting_agenda)
                    .sort_by(AgendaViewSortSpec::up(AgendaViewSortKey::Title)),
            ),
    );

    assert_eq!(block.title, "Daily agent agenda");
    assert_eq!(block.sections.len(), 2);
    assert_eq!(block.total_candidates, 3);
    assert_eq!(block.sections[0].index, 1);
    assert_eq!(block.sections[0].name, "Timed today");
    assert_eq!(block.sections[0].plan.cards[0].title, "Review queue");
    assert_eq!(block.sections[1].index, 2);
    assert_eq!(block.sections[1].name, "Waiting");
    assert_eq!(block.sections[1].plan.cards[0].title, "Review queue");
    assert!(block
        .to_compact_text("agenda.org")
        .contains("[AGENDA_SECTION] 2 Waiting"));
}
