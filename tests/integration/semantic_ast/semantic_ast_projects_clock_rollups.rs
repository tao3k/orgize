use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    ast::{
        ClockEffortStatus, ClockTableScopeKind, ClockTableTimeWindowSource, ClockTableWarningKind,
    },
    Org,
};

const SOURCE: &str = r#"#+BEGIN: clocktable :scope file :maxlevel 2
#+END:

* TODO Build API
:PROPERTIES:
:Effort: 2:00
:END:
:LOGBOOK:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:30] =>  1:30
:END:
** TODO Parser
:PROPERTIES:
:Effort: 0:45
:END:
CLOCK: [2026-05-15 Fri 10:30]--[2026-05-15 Fri 11:00] =>  0:30
*** TODO Hidden detail
:PROPERTIES:
:Effort: 0:15
:END:
CLOCK: [2026-05-15 Fri 11:00]--[2026-05-15 Fri 11:15] =>  0:15
* TODO Unclocked plan
:PROPERTIES:
:Effort: 0:30
:END:
"#;

#[test]
fn semantic_ast_projects_clock_rollups_for_effort_comparison() {
    let doc = Org::parse(SOURCE).document();
    assert_clean_projection(&doc);

    let records = doc.clock_rollup_records();
    assert_eq!(records.len(), 4);

    let build = records
        .iter()
        .find(|record| record.title == "Build API")
        .expect("build rollup");
    assert_eq!(build.local_clock.entries, 1);
    assert_eq!(build.local_clock.total_seconds, 5_400);
    assert_eq!(build.subtree_clock.entries, 3);
    assert_eq!(build.subtree_clock.total_seconds, 8_100);
    assert_eq!(build.effort.subtree_total_seconds, 10_800);
    assert_eq!(build.effort.delta_seconds, -2_700);
    assert_eq!(build.effort.status, ClockEffortStatus::UnderEffort);

    let parser = records
        .iter()
        .find(|record| record.title == "Parser")
        .expect("parser rollup");
    assert_eq!(parser.local_clock.total_seconds, 1_800);
    assert_eq!(parser.subtree_clock.total_seconds, 2_700);
    assert_eq!(parser.effort.status, ClockEffortStatus::UnderEffort);

    let unclocked = records
        .iter()
        .find(|record| record.title == "Unclocked plan")
        .expect("unclocked rollup");
    assert_eq!(unclocked.subtree_clock.total_seconds, 0);
    assert_eq!(unclocked.effort.subtree_total_seconds, 1_800);
}

#[test]
fn semantic_ast_projects_clocktable_plans_from_dynamic_blocks() {
    let doc = Org::parse(SOURCE).document();
    let plans = doc.clock_table_plans();

    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    assert_eq!(plan.name, "clocktable");
    assert_eq!(plan.scope.kind, ClockTableScopeKind::File);
    assert_eq!(plan.max_level, 2);
    assert_eq!(plan.rows.len(), 3);
    assert_eq!(plan.rows[0].title, "Build API");
    assert_eq!(plan.rows[0].table_level, 1);
    assert_eq!(plan.rows[0].clock.total_seconds, 8_100);
    assert_eq!(plan.rows[1].title, "Parser");
    assert_eq!(plan.rows[1].table_level, 2);
    assert_eq!(plan.rows[1].clock.total_seconds, 2_700);
    assert!(plan.rows.iter().all(|row| row.title != "Hidden detail"));
    assert_eq!(plan.rows[2].title, "Unclocked plan");
    assert_eq!(plan.rows[2].effort_total_seconds, 1_800);

    let compact = plan.to_compact_text("clock.org");
    assert!(compact.contains("[CLOCKTABLE] clocktable"));
    assert!(compact.contains("row: Build API | tableLevel=1 | clock=8100s/3 entries"));
    assert!(compact.contains("contract: Derived from official Org CLOCK"));
}

#[test]
fn semantic_ast_projects_clocktable_subtree_scope_and_absolute_time_params() {
    let doc = Org::parse(
        r#"* TODO Project
#+BEGIN: clocktable :scope subtree :maxlevel 1 :tstart "<2026-05-01 Thu>" :tend "<2026-05-31 Sun>"
#+END:
** TODO Child
:PROPERTIES:
:Effort: 1:00
:END:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 09:45] =>  0:45
* TODO Outside
:PROPERTIES:
:Effort: 3:00
:END:
CLOCK: [2026-05-15 Fri 10:00]--[2026-05-15 Fri 11:00] =>  1:00
"#,
    )
    .document();

    let plans = doc.clock_table_plans();
    assert_eq!(plans.len(), 1);
    let plan = &plans[0];
    assert_eq!(plan.scope.kind, ClockTableScopeKind::Subtree);
    assert_eq!(plan.tstart.as_deref(), Some("\"<2026-05-01 Thu>\""));
    assert_eq!(plan.tend.as_deref(), Some("\"<2026-05-31 Sun>\""));
    let time_window = plan.time_window.as_ref().expect("applied time window");
    assert_eq!(time_window.source, ClockTableTimeWindowSource::TstartTend);
    assert_eq!(time_window.start.expect("window start").day, 1);
    assert_eq!(
        time_window.end_exclusive.expect("window end").month,
        6,
        "date-only tend should include the whole end day"
    );
    assert_eq!(plan.rows.len(), 1);
    assert_eq!(plan.rows[0].title, "Project");
    assert_eq!(plan.rows[0].clock.total_seconds, 2_700);
    assert!(plan.rows.iter().all(|row| row.title != "Outside"));
    assert!(plan
        .warnings
        .iter()
        .all(|warning| warning.kind != ClockTableWarningKind::TimeRangePreserved));
}

#[test]
fn semantic_ast_projects_clocktable_clips_absolute_tstart_tend() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 2 :tstart "<2026-05-15 Fri 10:00>" :tend "<2026-05-15 Fri 11:00>"
#+END:

* TODO Build API
:PROPERTIES:
:Effort: 2:00
:END:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:30] =>  1:30
** TODO Parser
:PROPERTIES:
:Effort: 0:45
:END:
CLOCK: [2026-05-15 Fri 10:30]--[2026-05-15 Fri 11:00] =>  0:30
*** TODO Hidden detail
:PROPERTIES:
:Effort: 0:15
:END:
CLOCK: [2026-05-15 Fri 11:00]--[2026-05-15 Fri 11:15] =>  0:15
* TODO Unclocked plan
:PROPERTIES:
:Effort: 0:30
:END:
"#,
    )
    .document();

    let plans = doc.clock_table_plans();
    let plan = &plans[0];
    let time_window = plan.time_window.as_ref().expect("applied time window");
    assert_eq!(time_window.source, ClockTableTimeWindowSource::TstartTend);
    assert_eq!(time_window.start.expect("window start").hour, 10);
    assert_eq!(time_window.end_exclusive.expect("window end").hour, 11);
    assert_eq!(plan.rows.len(), 3);
    assert_eq!(plan.rows[0].title, "Build API");
    assert_eq!(plan.rows[0].clock.entries, 2);
    assert_eq!(plan.rows[0].clock.total_seconds, 3_600);
    assert_eq!(plan.rows[1].title, "Parser");
    assert_eq!(plan.rows[1].clock.entries, 1);
    assert_eq!(plan.rows[1].clock.total_seconds, 1_800);
    assert!(plan.rows.iter().all(|row| row.title != "Hidden detail"));
    assert_eq!(plan.rows[2].title, "Unclocked plan");
    assert_eq!(plan.rows[2].clock.total_seconds, 0);
}

#[test]
fn semantic_ast_projects_clocktable_expands_absolute_block_month() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 1 :block 2026-05
#+END:

* TODO Project
:PROPERTIES:
:Effort: 4:00
:END:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:00] =>  1:00
CLOCK: [2026-06-01 Mon 09:00]--[2026-06-01 Mon 11:00] =>  2:00
"#,
    )
    .document();

    let plan = &doc.clock_table_plans()[0];
    let time_window = plan.time_window.as_ref().expect("applied block window");
    assert_eq!(time_window.source, ClockTableTimeWindowSource::Block);
    assert_eq!(time_window.start.expect("window start").month, 5);
    assert_eq!(time_window.end_exclusive.expect("window end").month, 6);
    assert_eq!(plan.rows.len(), 1);
    assert_eq!(plan.rows[0].clock.entries, 1);
    assert_eq!(plan.rows[0].clock.total_seconds, 3_600);
    assert!(plan
        .warnings
        .iter()
        .all(|warning| warning.kind != ClockTableWarningKind::BlockRangePreserved));
}

#[test]
fn semantic_ast_projects_clocktable_preserves_relative_time_params() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 1 :tstart "<-1w>" :tend "<now>"
#+END:

* TODO Project
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:00] =>  1:00
"#,
    )
    .document();

    let plan = &doc.clock_table_plans()[0];
    assert!(plan.time_window.is_none());
    assert_eq!(plan.rows.len(), 1);
    assert_eq!(plan.rows[0].clock.total_seconds, 3_600);
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.kind == ClockTableWarningKind::TimeRangePreserved));
}

#[test]
fn semantic_ast_projects_clocktable_applies_match_filter_to_contributions() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 2 :match "+client-internal"
#+END:

* TODO Client project :client:
:PROPERTIES:
:Effort: 2:00
:END:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:00] =>  1:00
** TODO Internal detail :internal:
:PROPERTIES:
:Effort: 1:00
:END:
CLOCK: [2026-05-15 Fri 10:00]--[2026-05-15 Fri 11:00] =>  1:00
* TODO Client followup :client:
:PROPERTIES:
:Effort: 0:30
:END:
CLOCK: [2026-05-15 Fri 11:00]--[2026-05-15 Fri 11:30] =>  0:30
"#,
    )
    .document();

    let plan = &doc.clock_table_plans()[0];
    let match_filter = plan.match_filter.as_ref().expect("applied match filter");
    assert_eq!(match_filter.expression, "+client-internal");
    assert_eq!(plan.rows.len(), 2);
    assert_eq!(plan.rows[0].title, "Client project");
    assert_eq!(plan.rows[0].clock.entries, 1);
    assert_eq!(plan.rows[0].clock.total_seconds, 3_600);
    assert_eq!(plan.rows[0].effort_total_seconds, 7_200);
    assert!(plan.rows.iter().all(|row| row.title != "Internal detail"));
    assert_eq!(plan.rows[1].title, "Client followup");
    assert_eq!(plan.rows[1].clock.total_seconds, 1_800);
    assert!(plan
        .warnings
        .iter()
        .all(|warning| warning.kind != ClockTableWarningKind::MatchPreserved));
}

#[test]
fn semantic_ast_projects_clocktable_preserves_unparsed_match_filter() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 1 :match ""
#+END:

* TODO Project :client:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:00] =>  1:00
"#,
    )
    .document();

    let plan = &doc.clock_table_plans()[0];
    assert!(plan.match_filter.is_none());
    assert_eq!(plan.rows.len(), 1);
    assert_eq!(plan.rows[0].clock.total_seconds, 3_600);
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.kind == ClockTableWarningKind::MatchPreserved));
}

#[test]
fn semantic_ast_projects_clocktable_projects_property_columns() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 2 :properties ("Owner" "Phase") :inherit-props t
#+END:

* TODO Project
:PROPERTIES:
:Owner: Ada
:END:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:00] =>  1:00
** TODO Child
:PROPERTIES:
:Phase: Build
:END:
CLOCK: [2026-05-15 Fri 10:00]--[2026-05-15 Fri 11:00] =>  1:00
"#,
    )
    .document();

    let plan = &doc.clock_table_plans()[0];
    let columns = plan
        .property_columns
        .as_ref()
        .expect("applied property columns");
    assert_eq!(columns.names, ["Owner", "Phase"]);
    assert!(columns.inherit);
    assert_eq!(plan.rows.len(), 2);

    let project = &plan.rows[0];
    assert_eq!(project.title, "Project");
    assert_eq!(project.property_values.len(), 2);
    assert_eq!(project.property_values[0].name, "Owner");
    assert_eq!(project.property_values[0].value.as_deref(), Some("Ada"));
    assert!(!project.property_values[0].inherited);
    assert_eq!(project.property_values[1].name, "Phase");
    assert!(project.property_values[1].value.is_none());
    assert!(!project.property_values[1].inherited);

    let child = &plan.rows[1];
    assert_eq!(child.title, "Child");
    assert_eq!(child.property_values[0].value.as_deref(), Some("Ada"));
    assert!(child.property_values[0].inherited);
    assert_eq!(child.property_values[1].value.as_deref(), Some("Build"));
    assert!(!child.property_values[1].inherited);

    let compact = plan.to_compact_text("clock.org");
    assert!(compact.contains("properties: Owner, Phase inherit=true"));
}

#[test]
fn semantic_ast_projects_clocktable_preserves_unparsed_property_columns() {
    let doc = Org::parse(
        r#"#+BEGIN: clocktable :scope file :maxlevel 1 :properties Owner
#+END:

* TODO Project
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 10:00] =>  1:00
"#,
    )
    .document();

    let plan = &doc.clock_table_plans()[0];
    assert!(plan.property_columns.is_none());
    assert_eq!(plan.rows.len(), 1);
    assert_eq!(plan.rows[0].clock.total_seconds, 3_600);
    assert!(plan
        .warnings
        .iter()
        .any(|warning| warning.kind == ClockTableWarningKind::PropertiesPreserved));
}
