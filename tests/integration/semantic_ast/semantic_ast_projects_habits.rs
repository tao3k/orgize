use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{HabitConsistency, TimeUnit},
};

const SOURCE: &str = include_str!("../../fixtures/semantic_ast/habit-records.org");

#[test]
fn semantic_ast_projects_habit_metadata_for_agenda_consumers() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let habits = doc.habit_records();
    assert_eq!(habits.len(), 2);

    let daily = habits
        .iter()
        .find(|habit| habit.title == "Daily review")
        .expect("daily habit");
    assert_eq!(daily.consistency, HabitConsistency::Complete);
    assert_eq!(
        daily.effort.as_ref().map(|effort| effort.total_seconds),
        Some(1_800)
    );
    assert_eq!(daily.clock_count, 1);
    assert_eq!(daily.clock_total_seconds, 1_500);
    assert_eq!(
        daily
            .repeater
            .as_ref()
            .map(|repeater| (repeater.value, repeater.unit)),
        Some((1, TimeUnit::Day))
    );
    assert_eq!(
        daily.last_repeat.as_ref().map(|last| last.raw.as_str()),
        Some("[2026-05-14 Thu 08:00]")
    );

    let missing = habits
        .iter()
        .find(|habit| habit.title == "Missing repeater")
        .expect("missing repeater habit");
    assert_eq!(missing.consistency, HabitConsistency::MissingRepeater);

    insta::assert_debug_snapshot!("semantic_ast__semantic_habit_records", habits);
}
