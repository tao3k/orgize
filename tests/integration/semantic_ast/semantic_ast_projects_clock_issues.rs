use crate::semantic_ast::support::assert_clean_projection;
use orgize::{
    Org,
    ast::{ClockIssueFindingKind, ClockIssueProfile},
};

#[test]
fn semantic_ast_projects_clock_issue_findings_match_org_agenda_order() {
    let doc = Org::parse(
        r#"* TODO Work
:LOGBOOK:
CLOCK: [2026-05-15 Fri 09:00]--[2026-05-15 Fri 09:10] =>  0:10
CLOCK: [2026-05-15 Fri 09:05]--[2026-05-15 Fri 09:20] =>  0:15
CLOCK: [2026-05-15 Fri 09:40]--[2026-05-15 Fri 09:50] =>  0:10
CLOCK: [2026-05-15 Fri 10:00]--[2026-05-15 Fri 21:30] => 11:30
CLOCK: [2026-05-15 Fri 22:00]
CLOCK: [2026-05-16 Sat 01:00]--[2026-05-16 Sat 01:01] =>  0:01
:END:
"#,
    )
    .document();
    assert_clean_projection(&doc);

    let profile = ClockIssueProfile::org_default()
        .max_gap_seconds(10 * 60)
        .min_duration_seconds(2 * 60)
        .gap_ok_around_minutes(Vec::new());
    let findings = doc.clock_issue_findings_with_profile(&profile);
    let kinds = findings
        .iter()
        .map(|finding| finding.kind)
        .collect::<Vec<_>>();

    assert_eq!(
        kinds,
        [
            ClockIssueFindingKind::Overlap,
            ClockIssueFindingKind::Gap,
            ClockIssueFindingKind::LongDuration,
            ClockIssueFindingKind::NoEndTime,
            ClockIssueFindingKind::ShortDuration,
        ]
    );

    assert_eq!(findings[0].duration_seconds, Some(5 * 60));
    assert_eq!(
        findings[0]
            .previous_clock
            .as_ref()
            .unwrap()
            .duration_seconds,
        Some(10 * 60)
    );
    assert_eq!(findings[1].threshold_seconds, Some(10 * 60));
    assert_eq!(findings[2].duration_seconds, Some(11 * 60 * 60 + 30 * 60));
    assert_eq!(findings[2].threshold_seconds, Some(10 * 60 * 60));
    assert!(findings[3].clock.end.is_none());
    assert_eq!(findings[4].threshold_seconds, Some(2 * 60));

    let compact = findings[0].to_compact_text("clock.org");
    assert!(compact.contains("[CLOCK-ISSUE] overlap"));
    assert!(compact.contains("outline: Work"));
    assert!(compact.contains("contract: Derived from official Org CLOCK"));
}

#[test]
fn semantic_ast_projects_clock_issue_gap_ok_around_suppresses_default_night_gap() {
    let doc = Org::parse(
        r#"* TODO Work
CLOCK: [2026-05-15 Fri 17:00]--[2026-05-15 Fri 17:30] =>  0:30
CLOCK: [2026-05-16 Sat 09:00]--[2026-05-16 Sat 09:30] =>  0:30
"#,
    )
    .document();

    assert!(doc.clock_issue_findings().is_empty());

    let strict = ClockIssueProfile::org_default().gap_ok_around_minutes(Vec::new());
    let findings = doc.clock_issue_findings_with_profile(&strict);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, ClockIssueFindingKind::Gap);
    assert_eq!(findings[0].duration_seconds, Some(15 * 60 * 60 + 30 * 60));
}
