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

    assert_eq!(entries.len(), 11);
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
        .find(|entry| entry.raw_title == "Delayed scheduled 8:30-1pm")
        .expect("scheduled delay row");
    assert_eq!(delayed.display_date, AgendaDate::new(2026, 5, 16));
    assert_eq!(
        delayed.scheduled,
        Some(AgendaScheduleState::Delayed { days_delayed: 2 })
    );
    assert_eq!(
        delayed.time,
        Some(AgendaTime {
            hour: 8,
            minute: 30
        })
    );
    assert_eq!(
        delayed.end_time,
        Some(AgendaTime {
            hour: 13,
            minute: 0
        })
    );

    let plain_meetings = entries
        .iter()
        .filter(|entry| entry.raw_title == "Plain meeting")
        .collect::<Vec<_>>();
    assert_eq!(plain_meetings.len(), 1);
    assert_eq!(plain_meetings[0].kind, AgendaEntryKind::Timestamp);
    assert_eq!(
        plain_meetings[0].time,
        Some(AgendaTime {
            hour: 14,
            minute: 0
        })
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
        .exclude_tag("delay")
        .exclude_tag("event");

    let titles = doc
        .to_bare()
        .agenda_entries(&query)
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();

    assert_eq!(titles, ["Archived item", "Done item"]);
}

#[test]
fn semantic_ast_agenda_filters_with_official_match_expression_subset() {
    let doc = Org::parse(
        r#"#+TODO: TODO WAITING | DONE
#+FILETAGS: :team:
#+CATEGORY: inbox
* WAITING Work wait :work:
SCHEDULED: <2026-05-15 Fri>
:PROPERTIES:
:Effort: 1
:Owner: Sarah
:END:
* TODO Home larger :home:
SCHEDULED: <2026-05-15 Fri>
:PROPERTIES:
:Effort: 3
:Owner: Taylor
:END:
* DONE Work done :work:
SCHEDULED: <2026-05-15 Fri>
"#,
    )
    .document();
    let query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 15))
        .include_done(true)
        .match_expression(r#"+work+TODO="WAITING"|home+Effort<2"#)
        .expect("valid agenda match expression");

    let match_entries = doc.to_bare().agenda_entries(&query);
    let titles = match_entries
        .iter()
        .map(|entry| entry.raw_title.as_str())
        .collect::<Vec<_>>();

    assert_eq!(titles, ["Work wait"]);
    insta::with_settings!({snapshot_path => "../../snapshots", prepend_module_to_snapshot => false}, {
        insta::assert_debug_snapshot!("semantic_ast__semantic_agenda_match_filter_entries", match_entries);
    });

    let category_query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 15))
        .include_done(true)
        .match_expression(r#"+team+CATEGORY="inbox"-Owner="Taylor""#)
        .expect("valid agenda match expression");
    let titles = doc
        .to_bare()
        .agenda_entries(&category_query)
        .into_iter()
        .map(|entry| entry.raw_title)
        .collect::<Vec<_>>();

    assert_eq!(titles, ["Work done", "Work wait"]);
}

#[test]
fn semantic_ast_agenda_projects_active_timestamps_only() {
    let doc = Org::parse("* TODO Event\nBody <2026-05-15 Fri 08:00> and [2026-05-15 Fri 09:00].\n")
        .document();
    let query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 15));

    let entries = doc.to_bare().agenda_entries(&query);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, AgendaEntryKind::Timestamp);
    assert_eq!(entries[0].time, Some(AgendaTime { hour: 8, minute: 0 }));

    let without_timestamps = doc
        .to_bare()
        .agenda_entries(&query.include_timestamps(false));
    assert!(without_timestamps.is_empty());
}

#[test]
fn semantic_ast_agenda_extracts_headline_time_of_day() {
    let doc = Org::parse(
        "* TODO Arthur Dent 8:30-1pm\nSCHEDULED: <2026-05-15 Fri>\n* TODO Ford <2026-05-15 Fri 12:45>\n",
    )
    .document();
    let query = AgendaQuery::single_day(AgendaDate::new(2026, 5, 15));

    let entries = doc.to_bare().agenda_entries(&query);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].raw_title, "Arthur Dent 8:30-1pm");
    assert_eq!(entries[0].kind, AgendaEntryKind::Scheduled);
    assert_eq!(
        entries[0].time,
        Some(AgendaTime {
            hour: 8,
            minute: 30
        })
    );
    assert_eq!(
        entries[0].end_time,
        Some(AgendaTime {
            hour: 13,
            minute: 0
        })
    );
    assert_eq!(entries[1].raw_title, "Ford <2026-05-15 Fri 12:45>");
    assert_eq!(entries[1].kind, AgendaEntryKind::Timestamp);
    assert_eq!(
        entries[1].time,
        Some(AgendaTime {
            hour: 12,
            minute: 45
        })
    );

    let without_headline_time = doc
        .to_bare()
        .agenda_entries(&query.search_headline_time(false));
    let arthur = without_headline_time
        .iter()
        .find(|entry| entry.raw_title == "Arthur Dent 8:30-1pm")
        .expect("scheduled row");
    assert_eq!(arthur.time, None);
    assert_eq!(arthur.end_time, None);
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
