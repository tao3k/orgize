use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        AgendaDate, AgendaDeadlineState, AgendaEntryKind, AgendaOccurrence, AgendaQuery,
        AgendaScheduleState, AgendaTime,
    },
    Org,
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/agenda-planning.org");

#[test]
fn semantic_ast_projects_planning_timestamps_to_agenda_entries() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let query = AgendaQuery::new(AgendaDate::new(2026, 5, 14), AgendaDate::new(2026, 5, 16))
        .include_closed(true);

    let bare = doc.to_bare();
    let entries = bare.agenda_entries(&query);

    assert_eq!(entries.len(), 10);
    assert_eq!(entries[0].kind, AgendaEntryKind::Deadline);
    assert_eq!(entries[0].raw_title, "Deadline warning");
    assert_eq!(
        entries[0]
            .category
            .as_ref()
            .map(|category| category.as_str()),
        Some("doc-cat")
    );
    assert_eq!(entries[0].display_date, AgendaDate::new(2026, 5, 14));
    assert_eq!(entries[0].target_date, AgendaDate::new(2026, 5, 16));
    assert_eq!(
        entries[0].deadline,
        Some(AgendaDeadlineState::Warning { days_until: 2 })
    );
    let repeated = entries
        .iter()
        .find(|entry| {
            entry.raw_title == "Scheduled daily"
                && entry.target_date == AgendaDate::new(2026, 5, 15)
        })
        .expect("daily scheduled repeater occurrence");
    assert_eq!(repeated.occurrence, AgendaOccurrence::Repeater { index: 1 });
    assert_eq!(
        repeated.category.as_ref().map(|category| category.as_str()),
        Some("work-cat")
    );

    let range_entries = entries
        .iter()
        .filter(|entry| entry.raw_title == "Range event")
        .collect::<Vec<_>>();
    assert_eq!(range_entries.len(), 2);
    assert_eq!(range_entries[0].display_date, AgendaDate::new(2026, 5, 14));
    assert_eq!(range_entries[1].display_date, AgendaDate::new(2026, 5, 15));
    assert_eq!(
        range_entries[0].target_end_date,
        Some(AgendaDate::new(2026, 5, 15))
    );
    assert_eq!(
        range_entries[0].time,
        Some(AgendaTime {
            hour: 10,
            minute: 0
        })
    );
    assert_eq!(
        range_entries[0].end_time,
        Some(AgendaTime {
            hour: 11,
            minute: 0
        })
    );

    let delayed = entries
        .iter()
        .find(|entry| entry.raw_title == "Delayed scheduled")
        .expect("scheduled delay row");
    assert_eq!(delayed.display_date, AgendaDate::new(2026, 5, 16));
    assert_eq!(
        delayed.scheduled,
        Some(AgendaScheduleState::Delayed { days_delayed: 2 })
    );

    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_agenda_entries", entries);
    });
}

#[test]
fn semantic_ast_agenda_filters_done_archived_and_tags() {
    let doc = Org::parse(SOURCE).document();
    let query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 14))
        .include_done(true)
        .include_archived(true)
        .exclude_tag("work")
        .exclude_tag("ops")
        .exclude_tag("range")
        .exclude_tag("delay");

    let titles = doc
        .to_bare()
        .agenda_entries(&query)
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();

    assert_eq!(titles, ["Archived item", "Done item"]);
}

#[test]
fn semantic_ast_agenda_reports_overdue_deadlines_on_window_start() {
    let doc = Org::parse("* TODO Late\nDEADLINE: <2026-05-10 Sun>\n").document();
    let entries = doc
        .to_bare()
        .agenda_entries(&AgendaQuery::single_day(AgendaDate::new(2026, 5, 14)));

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, AgendaEntryKind::Deadline);
    assert_eq!(entries[0].display_date, AgendaDate::new(2026, 5, 14));
    assert_eq!(entries[0].target_date, AgendaDate::new(2026, 5, 10));
    assert_eq!(
        entries[0].deadline,
        Some(AgendaDeadlineState::Overdue { days_overdue: 4 })
    );
}
